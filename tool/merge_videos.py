#!/usr/bin/env python3
"""
Video Merger Tool for Ruscut
---------------------------
A fast and robust command-line utility to concatenate/merge multiple video files using FFmpeg.
If input videos have identical properties (codec, dimensions, frame rate, audio), 
it merges them losslessly without re-encoding in a fraction of a second.
Otherwise, it automatically falls back to re-encoding using high-quality H.264.
"""

import os
import sys
import json
import argparse
import subprocess
import tempfile

# ANSI escape codes for stylish terminal outputs
BOLD = "\033[1m"
GREEN = "\033[32m"
BLUE = "\033[34m"
YELLOW = "\033[33m"
RED = "\033[31m"
RESET = "\033[0m"

def print_step(msg):
    print(f"{BLUE}{BOLD}[*]{RESET} {msg}")

def print_success(msg):
    print(f"{GREEN}{BOLD}[✓]{RESET} {msg}")

def print_warning(msg):
    print(f"{YELLOW}{BOLD}[!]{RESET} {msg}")

def print_error(msg):
    print(f"{RED}{BOLD}[✗]{RESET} {msg}", file=sys.stderr)

def check_dependencies():
    """Ensure ffmpeg and ffprobe are available on the path."""
    for tool in ['ffmpeg', 'ffprobe']:
        try:
            subprocess.run([tool, '-version'], capture_output=True, check=True)
        except (subprocess.CalledProcessError, FileNotFoundError):
            print_error(f"Required dependency '{tool}' is not installed or not in PATH.")
            sys.exit(1)

def probe_video(filepath):
    """Probes a video file using ffprobe and extracts technical specifications."""
    if not os.path.exists(filepath):
        raise FileNotFoundError(f"File not found: {filepath}")
    
    cmd = [
        'ffprobe', '-v', 'error',
        '-show_entries', 'stream=codec_name,codec_type,width,height,r_frame_rate',
        '-of', 'json', filepath
    ]
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        data = json.loads(result.stdout)
    except subprocess.CalledProcessError as e:
        raise RuntimeError(f"ffprobe failed for {filepath}: {e.stderr}")
    except json.JSONDecodeError:
        raise RuntimeError(f"Failed to parse ffprobe metadata for {filepath}")

    info = {
        'path': filepath,
        'has_video': False,
        'has_audio': False,
        'width': None,
        'height': None,
        'video_codec': None,
        'frame_rate': None,
        'audio_codec': None
    }

    for stream in data.get('streams', []):
        stype = stream.get('codec_type')
        if stype == 'video':
            info['has_video'] = True
            info['width'] = stream.get('width')
            info['height'] = stream.get('height')
            info['video_codec'] = stream.get('codec_name')
            info['frame_rate'] = stream.get('r_frame_rate')
        elif stype == 'audio':
            info['has_audio'] = True
            info['audio_codec'] = stream.get('codec_name')

    return info

def check_compatibility(video_infos):
    """
    Checks if all videos have matching streams, codecs, resolutions, and framerates,
    which permits fast-copy concatenation without re-encoding.
    """
    if not video_infos:
        return True
    
    first = video_infos[0]
    
    # Basic check for presence of video stream
    if not first['has_video']:
        print_warning(f"File {first['path']} does not contain a video stream.")
        return False

    for info in video_infos[1:]:
        if not info['has_video']:
            print_warning(f"File {info['path']} does not contain a video stream.")
            return False
        if info['width'] != first['width'] or info['height'] != first['height']:
            print_warning(f"Resolution mismatch: {first['path']} ({first['width']}x{first['height']}) vs {info['path']} ({info['width']}x{info['height']})")
            return False
        if info['video_codec'] != first['video_codec']:
            print_warning(f"Codec mismatch: {first['path']} ({first['video_codec']}) vs {info['path']} ({info['video_codec']})")
            return False
        if info['frame_rate'] != first['frame_rate']:
            print_warning(f"Frame rate mismatch: {first['path']} ({first['frame_rate']}) vs {info['path']} ({info['frame_rate']})")
            return False
        if info['has_audio'] != first['has_audio']:
            print_warning(f"Audio presence mismatch: {first['path']} (has_audio={first['has_audio']}) vs {info['path']} (has_audio={info['has_audio']})")
            return False
            
    return True

def concat_fast(video_paths, output_path):
    """Concatenates videos instantly using the FFmpeg concat demuxer (lossless copy)."""
    print_step("Performing fast lossless concatenation (no re-encoding)...")
    
    # We must write absolute paths to the concat list file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
        list_file_path = f.name
        for path in video_paths:
            abs_path = os.path.abspath(path)
            # Escape single quotes for ffmpeg concat file format
            escaped_path = abs_path.replace("'", "'\\''")
            f.write(f"file '{escaped_path}'\n")
            
    cmd = [
        'ffmpeg', '-y',
        '-f', 'concat',
        '-safe', '0',
        '-i', list_file_path,
        '-c', 'copy',
        '-movflags', '+faststart',
        output_path
    ]
    
    try:
        print_step("Running FFmpeg concat command...")
        process = subprocess.run(cmd, capture_output=True, text=True)
        if process.returncode != 0:
            print_error(f"FFmpeg failed with exit code {process.returncode}")
            print_error(process.stderr)
            sys.exit(1)
    finally:
        if os.path.exists(list_file_path):
            os.remove(list_file_path)

def concat_reencode(video_infos, output_path):
    """Concatenates videos by decoding and re-encoding them."""
    print_warning("Videos are not perfectly matched. Falling back to re-encoding...")
    print_step("Merging with high-quality H.264/AAC re-encoding (this might take a moment)...")
    
    num_inputs = len(video_infos)
    all_have_audio = all(info['has_audio'] for info in video_infos)
    
    cmd = ['ffmpeg', '-y']
    for info in video_infos:
        cmd.extend(['-i', info['path']])
        
    filter_complex = ""
    for i in range(num_inputs):
        filter_complex += f"[{i}:v]"
        if all_have_audio:
            filter_complex += f"[{i}:a]"
            
    a_flag = "1" if all_have_audio else "0"
    filter_complex += f"concat=n={num_inputs}:v=1:a={a_flag}[outv]"
    if all_have_audio:
        filter_complex += "[outa]"
        
    cmd.extend(['-filter_complex', filter_complex])
    cmd.extend(['-map', '[outv]'])
    if all_have_audio:
        cmd.extend(['-map', '[outa]'])
        
    # Standard high-compatibility encoding profiles
    cmd.extend([
        '-c:v', 'libx264',
        '-pix_fmt', 'yuv420p',
        '-preset', 'medium',
        '-crf', '21',
    ])
    
    if all_have_audio:
        cmd.extend([
            '-c:a', 'aac',
            '-b:a', '192k'
        ])
        
    cmd.extend([
        '-movflags', '+faststart',
        output_path
    ])
    
    print_step("Running FFmpeg re-encode command...")
    process = subprocess.run(cmd, capture_output=True, text=True)
    if process.returncode != 0:
        print_error(f"FFmpeg failed with exit code {process.returncode}")
        print_error(process.stderr)
        sys.exit(1)

def main():
    check_dependencies()
    
    parser = argparse.ArgumentParser(
        description="Merge/concatenate multiple videos using FFmpeg.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  ./merge_videos.py video1.mp4 video2.mp4 -o output.mp4
  ./merge_videos.py part1.mp4 part2.mp4 part3.mp4 --force-reencode
        """
    )
    parser.add_argument('inputs', nargs='+', help="Input video files to merge")
    parser.add_argument('-o', '--output', default='merged.mp4', help="Output merged video file (default: merged.mp4)")
    parser.add_argument('-f', '--force-reencode', action='store_true', help="Force re-encoding even if videos are compatible")
    
    args = parser.parse_args()
    
    if len(args.inputs) < 2:
        print_error("Error: You must provide at least 2 input videos to merge.")
        sys.exit(1)
        
    print_step(f"Analyzing {len(args.inputs)} video files...")
    
    video_infos = []
    for path in args.inputs:
        try:
            info = probe_video(path)
            video_infos.append(info)
            audio_str = f"with audio ({info['audio_codec']})" if info['has_audio'] else "no audio"
            print(f"  - {BLUE}{path}{RESET}: {BOLD}{info['width']}x{info['height']}{RESET} | {info['video_codec']} | {info['frame_rate']} | {audio_str}")
        except Exception as e:
            print_error(str(e))
            sys.exit(1)
            
    is_compatible = check_compatibility(video_infos)
    
    if is_compatible and not args.force_reencode:
        print_success("Videos have matching parameters. Fast-copy is available!")
        concat_fast(args.inputs, args.output)
    else:
        if args.force_reencode:
            print_step("Force re-encoding was explicitly requested.")
        concat_fast_possible = False
        concat_reencode(video_infos, args.output)
        
    print_success(f"Successfully merged video files into: {BOLD}{args.output}{RESET}")

if __name__ == '__main__':
    main()
