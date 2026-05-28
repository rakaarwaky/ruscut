use crate::contract::VideoProcessorPort;
use crate::taxonomy::EngineNameVo;
use anyhow::Context;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Guard to guarantee temporary directories are cleaned up even on early return/panic.
struct TempDirGuard {
    path: PathBuf,
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[derive(Default)]
pub struct FfmpegVideoAdapter {
    _initialized: bool,
}

impl FfmpegVideoAdapter {
    pub fn new() -> Self {
        Self { _initialized: true }
    }

    pub fn engine_name(&self) -> EngineNameVo {
        EngineNameVo::new("FFmpeg")
    }

    fn check_ffmpeg_available(&self) -> anyhow::Result<()> {
        let ffmpeg_check = Command::new("ffmpeg").arg("-version").output();
        if ffmpeg_check.is_err() {
            anyhow::bail!(
                "FFmpeg is not installed or not found in your PATH.\n\n\
                 {} To process video files, please install FFmpeg:\n\
                 - Linux (Ubuntu/Debian): {}\n\
                 - Linux (Fedora/RHEL): {}\n\
                 - macOS: {}\n\
                 - Windows: Download from {}",
                "ERROR:".red().bold(),
                "sudo apt install ffmpeg".yellow(),
                "sudo dnf install ffmpeg".yellow(),
                "brew install ffmpeg".yellow(),
                "https://ffmpeg.org".underline()
            );
        }
        Ok(())
    }

    fn get_video_fps(&self, input_path: &Path) -> f32 {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=avg_frame_rate",
                "-of",
                "default=noprint_wrappers=1:noclose=1=1",
                &input_path.to_string_lossy(),
            ])
            .output();

        if let Ok(out) = output
            && out.status.success()
        {
            let out_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Some((num_str, den_str)) = out_str.split_once('/') {
                let num = num_str.parse::<f32>().unwrap_or(30.0);
                let den = den_str.parse::<f32>().unwrap_or(1.0);
                if den > 0.0 {
                    return num / den;
                }
            } else if let Ok(fps) = out_str.parse::<f32>() {
                return fps;
            }
        }
        30.0
    }
}

impl VideoProcessorPort for FfmpegVideoAdapter {
    fn video_process_video(
        &self,
        input_path: &Path,
        output_path: &Path,
        process_frame: &dyn Fn(&Path, &Path) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        self.check_ffmpeg_available()?;

        let start_time = std::time::Instant::now();
        let fps = self.get_video_fps(input_path);
        println!(
            "{} Detected framerate: {} FPS",
            "INFO:".blue().bold(),
            format!("{:.2}", fps).yellow()
        );

        // Setup temporary directory
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let temp_dir = std::env::temp_dir().join(format!("ruscut_video_{}", timestamp));
        std::fs::create_dir_all(&temp_dir)
            .context("Failed to create temporary directory for video frames")?;
        let _guard = TempDirGuard {
            path: temp_dir.clone(),
        };

        // 1. Extract frames from input video
        println!("{} Extracting frames from video...", "INFO:".blue().bold());
        let extract_status = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                &input_path.to_string_lossy(),
                "-vf",
                &format!("fps={}", fps),
                &temp_dir.join("frame_%04d.png").to_string_lossy(),
            ])
            .status()
            .context("Failed to execute ffmpeg for frame extraction")?;

        if !extract_status.success() {
            anyhow::bail!("FFmpeg frame extraction failed");
        }

        // 2. Gather extracted frames
        let mut entries = std::fs::read_dir(&temp_dir)?
            .filter_map(|res| res.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.is_file()
                    && path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .starts_with("frame_")
            })
            .collect::<Vec<_>>();

        if entries.is_empty() {
            anyhow::bail!("No frames extracted from the video");
        }
        entries.sort();

        let total_frames = entries.len();
        println!(
            "{} Processing {} frames using BRIA RMBG-2.0...",
            "INFO:".blue().bold(),
            total_frames.to_string().yellow()
        );

        // Determine if we are outputting a PNG sequence (directory) or WebM (file)
        let is_sequence = if let Some(ext) = output_path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            ext_str != "webm" && ext_str != "mp4" && ext_str != "mov"
        } else {
            true // No extension = Default to PNG sequence directory
        };

        let pb = ProgressBar::new(total_frames as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} frames ({percent}%) - ETA: {eta}")?
            .progress_chars("#>-"));

        if is_sequence {
            std::fs::create_dir_all(output_path)
                .context("Failed to create output directory for PNG sequence")?;

            for (i, frame_path) in entries.iter().enumerate() {
                let target_filename = format!("frame_{:04}.png", i + 1);
                let target_path = output_path.join(target_filename);
                process_frame(frame_path, &target_path)?;
                pb.set_position((i + 1) as u64);
            }
            pb.finish_and_clear();

            let duration = start_time.elapsed();
            println!(
                "\n{} Successfully processed video in {:.2?}!",
                "SUCCESS:".green().bold(),
                duration
            );
            println!(
                "{} PNG Image Sequence saved to folder: {}",
                "OUTPUT:".green().bold(),
                output_path.to_string_lossy().underline()
            );
        } else {
            let processed_dir = temp_dir.join("processed");
            std::fs::create_dir_all(&processed_dir)
                .context("Failed to create processed frame directory")?;

            for (i, frame_path) in entries.iter().enumerate() {
                let target_path = processed_dir.join(format!("frame_{:04}.png", i + 1));
                process_frame(frame_path, &target_path)?;
                pb.set_position((i + 1) as u64);
            }
            pb.finish_and_clear();

            println!(
                "{} Encoding transparent video to WebM...",
                "INFO:".blue().bold()
            );
            // Compile back to transparent WebM with audio copy if it exists
            let encode_status = Command::new("ffmpeg")
                .args([
                    "-y",
                    "-f",
                    "image2",
                    "-framerate",
                    &fps.to_string(),
                    "-i",
                    &processed_dir.join("frame_%04d.png").to_string_lossy(),
                    "-i",
                    &input_path.to_string_lossy(),
                    "-map",
                    "0:v",
                    "-map",
                    "1:a?",
                    "-c:v",
                    "libvpx-vp9",
                    "-pix_fmt",
                    "yuva420p",
                    "-c:a",
                    "libvorbis",
                    "-shortest",
                    &output_path.to_string_lossy(),
                ])
                .status()
                .context("Failed to execute ffmpeg for transparent video encoding")?;

            if !encode_status.success() {
                anyhow::bail!(
                    "FFmpeg encoding failed. WebM VP9 might not be fully supported by this installation."
                );
            }

            let duration = start_time.elapsed();
            println!(
                "\n{} Successfully processed video in {:.2?}!",
                "SUCCESS:".green().bold(),
                duration
            );
            println!(
                "{} Transparent WebM video saved to: {}",
                "OUTPUT:".green().bold(),
                output_path.to_string_lossy().underline()
            );
        }

        Ok(())
    }
}
