#!/usr/bin/env python3
"""
Video Upscaler Tool for Ruscut
-----------------------------
A highly advanced and professional command-line utility to upscale videos using FFmpeg.
Offers premium algorithmic upscaling (Lanczos, Bicubic) combined with intelligent 
post-processing filters (sharpening, denoising) for stellar visual enhancements.
"""

import os
import sys
import json
import argparse
import subprocess

# ANSI escape codes for stylish terminal outputs
BOLD = "\033[1m"
GREEN = "\033[32m"
BLUE = "\033[34m"
YELLOW = "\033[33m"
RED = "\033[31m"
RESET = "\033[0m"

# Standard target resolutions
RESOLUTIONS = {
    '720p': (1280, 720),
    '1080p': (1920, 1080),
    '2k': (2560, 1440),
    '4k': (3840, 2160),
}

def print_step(msg):
    print(f"{BLUE}{BOLD}[*]{RESET} {msg}")

def print_success(msg):
    print(f"{GREEN}{BOLD}[✓]{RESET} {msg}")

def print_warning(msg):
    print(f"{YELLOW}{BOLD}[!]{RESET} {msg}")

def print_error(msg):
    print(f"{RED}{BOLD}[✗]{RESET} {msg}", file=sys.stderr)

def check_dependencies():
    """Ensure ffmpeg and ffprobe are available in the system PATH."""
    for tool in ['ffmpeg', 'ffprobe']:
        try:
            subprocess.run([tool, '-version'], capture_output=True, check=True)
        except (subprocess.CalledProcessError, FileNotFoundError):
            print_error(f"Required dependency '{tool}' is not installed or not in PATH.")
            sys.exit(1)

def probe_video(filepath):
    """Probes video properties using ffprobe."""
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
        'width': None,
        'height': None,
        'video_codec': None,
        'frame_rate': None
    }

    for stream in data.get('streams', []):
        stype = stream.get('codec_type')
        if stype == 'video':
            info['has_video'] = True
            info['width'] = stream.get('width')
            info['height'] = stream.get('height')
            info['video_codec'] = stream.get('codec_name')
            info['frame_rate'] = stream.get('r_frame_rate')
            break # primary video stream

    return info

def build_filter_graph(info, args, target_w, target_h):
    """Assembles the complex FFmpeg filtergraph for upscale, enhancement, and denoising."""
    filters = []
    
    # 1. Denoising: Remove noise and compression artifacts *before* upscaling
    if args.denoise:
        # hqdn3d is an excellent temporal-spatial denoiser
        filters.append("hqdn3d=1.5:1.5:6:6")
        
    # 2. Scaling and Aspect Ratio handling
    if args.scale_factor:
        filters.append(f"scale=iw*{args.scale_factor}:ih*{args.scale_factor}:flags={args.algo}")
    else:
        w, h = target_w, target_h
        if args.aspect == 'keep':
            # Scale to fit inside the target dimensions and pad the rest with black bars
            filters.append(f"scale=w={w}:h={h}:force_original_aspect_ratio=decrease:flags={args.algo}")
            filters.append(f"pad=w={w}:h={h}:x=(ow-iw)/2:y=(oh-ih)/2:color=black")
        elif args.aspect == 'crop':
            # Scale to completely fill the target dimensions, cropping the excess
            filters.append(f"scale=w={w}:h={h}:force_original_aspect_ratio=increase:flags={args.algo}")
            filters.append(f"crop=w={w}:h={h}")
        else:
            # Stretch video to exactly match target width/height
            filters.append(f"scale=w={w}:h={h}:flags={args.algo}")
            
    # 3. Intelligent Sharpening: Bring out edges and textures *after* upscaling
    if args.sharpen:
        # Fine-tuned unsharp mask: standard threshold, light amount to prevent haloing
        filters.append("unsharp=5:5:0.7:5:5:0.0")
        
    return ",".join(filters)

def main():
    check_dependencies()
    
    parser = argparse.ArgumentParser(
        description="Premium Video Upscaler using FFmpeg.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Upscale a video to 1080p (Lanczos + Denoise + Sharpen)
  ./upscale_video.py input.mp4 -t 1080p --sharpen --denoise -o upscaled_1080p.mp4

  # Scale a video to exactly 2x size using nearest-neighbor (good for pixel art)
  ./upscale_video.py input.mp4 -f 2 -a neighbor -o pixel_art_2x.mp4
        """
    )
    parser.add_argument('input', help="Input video file to upscale")
    parser.add_argument('-o', '--output', help="Output file path (default: <input>_upscaled.mp4)")
    
    # Target options
    target_group = parser.add_mutually_exclusive_group(required=True)
    target_group.add_argument('-t', '--target', choices=RESOLUTIONS.keys(), help="Target standard resolution (720p, 1080p, 2k, 4k)")
    target_group.add_argument('-f', '--scale-factor', type=float, help="Scale multiplier factor (e.g. 1.5, 2.0, 4.0)")
    target_group.add_argument('-c', '--custom', help="Custom dimensions as WIDTHxHEIGHT (e.g. 1920x1080)")
    
    # Scaling algorithm
    parser.add_argument('-a', '--algo', choices=['lanczos', 'bicubic', 'neighbor', 'bilinear'], default='lanczos',
                        help="Scaling algorithm (default: lanczos - best quality for realistic video)")
    
    # Aspect Ratio behavior (only applies if target resolution or custom is set)
    parser.add_argument('--aspect', choices=['keep', 'crop', 'stretch'], default='keep',
                        help="Aspect ratio adjustment mode (default: keep - pad with letterbox/pillarbox)")
    
    # Enhancements
    parser.add_argument('-s', '--sharpen', action='store_true', help="Apply intelligent post-scale sharpening (unsharp mask)")
    parser.add_argument('-d', '--denoise', action='store_true', help="Apply high-quality pre-scale denoising to clean up compression artifacts")
    
    # Encoding profile controls
    parser.add_argument('--crf', type=int, default=18, help="H.264 CRF quality factor (0-51, lower is better quality, default: 18)")
    parser.add_argument('--preset', default='slow', choices=['ultrafast', 'superfast', 'veryfast', 'faster', 'fast', 'medium', 'slow', 'slower', 'veryslow'],
                        help="FFmpeg encoder speed preset (default: slow)")

    args = parser.parse_args()
    
    # Verify input exists
    if not os.path.exists(args.input):
        print_error(f"Input file not found: {args.input}")
        sys.exit(1)
        
    print_step(f"Analyzing input video: {BOLD}{args.input}{RESET}...")
    try:
        info = probe_video(args.input)
        if not info['has_video']:
            print_error("The input file does not contain a valid video stream.")
            sys.exit(1)
        print(f"  - Original Resolution: {BOLD}{info['width']}x{info['height']}{RESET}")
        print(f"  - Codec: {info['video_codec']} | Framerate: {info['frame_rate']}")
    except Exception as e:
        print_error(str(e))
        sys.exit(1)
        
    # Resolve target dimensions
    target_w, target_h = None, None
    target_desc = ""
    
    if args.scale_factor:
        target_w = int(info['width'] * args.scale_factor)
        target_h = int(info['height'] * args.scale_factor)
        target_desc = f"{args.scale_factor}x factor ({target_w}x{target_h})"
    elif args.target:
        target_w, target_h = RESOLUTIONS[args.target]
        target_desc = f"{args.target} ({target_w}x{target_h})"
    elif args.custom:
        try:
            parts = args.custom.lower().split('x')
            target_w = int(parts[0])
            target_h = int(parts[1])
            target_desc = f"custom ({target_w}x{target_h})"
        except Exception:
            print_error("Custom size must be in the format WIDTHxHEIGHT (e.g. 1920x1080)")
            sys.exit(1)
            
    # Resolve output path
    output_path = args.output
    if not output_path:
        base, ext = os.path.splitext(args.input)
        output_path = f"{base}_upscaled{ext}"
        
    # Build filter graph
    filter_graph = build_filter_graph(info, args, target_w, target_h)
    
    print_step(f"Upscaling config:")
    print(f"  - Target: {BOLD}{target_desc}{RESET}")
    print(f"  - Algorithm: {BOLD}{args.algo}{RESET}")
    print(f"  - Aspect Ratio Mode: {args.aspect}")
    print(f"  - Denoise: {'Yes' if args.denoise else 'No'}")
    print(f"  - Sharpen: {'Yes' if args.sharpen else 'No'}")
    print(f"  - Output: {BOLD}{output_path}{RESET}")
    
    # Build FFmpeg command
    cmd = [
        'ffmpeg', '-y',
        '-i', args.input,
        '-vf', filter_graph,
        '-c:v', 'libx264',
        '-preset', args.preset,
        '-crf', str(args.crf),
        '-pix_fmt', 'yuv420p',
        '-c:a', 'copy', # Losslessly copy audio
        output_path
    ]
    
    print_step("Running FFmpeg to upscale video. Please wait...")
    try:
        # Run subprocess with stdout/stderr pipe or live output
        process = subprocess.run(cmd, capture_output=True, text=True)
        if process.returncode != 0:
            print_error(f"FFmpeg upscaling failed with exit code {process.returncode}")
            print_error(process.stderr)
            sys.exit(1)
            
        print_success(f"Successfully upscaled video! Saved to: {BOLD}{output_path}{RESET}")
    except KeyboardInterrupt:
        print_warning("\nUpscaling process interrupted by user.")
        sys.exit(1)

if __name__ == '__main__':
    main()
