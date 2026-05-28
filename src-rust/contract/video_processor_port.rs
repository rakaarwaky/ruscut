use std::path::Path;

/// Outbound port for executing background removal on video formats.
pub trait VideoProcessorPort: Send + Sync {
    fn video_process_video(
        &self,
        input_path: &Path,
        output_path: &Path,
        process_frame: &dyn Fn(&Path, &Path) -> anyhow::Result<()>,
    ) -> anyhow::Result<()>;
}
