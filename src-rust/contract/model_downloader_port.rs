use std::path::PathBuf;
use crate::taxonomy::removal_types_vo::ModelType;

/// Outbound port for downloading and ensuring model ONNX files.
/// This defines the formal boundary for fetching technical model assets.
///
/// # Implementations
/// - HuggingfaceModelAdapter (in Infrastructure layer)
///
/// # Safety
/// Trait requires Send and Sync constraints for safe concurrent operations.
pub trait ModelDownloaderPort: Send + Sync {
    fn downloader_ensure_model(&self, model_type: &ModelType, force: bool) -> anyhow::Result<PathBuf>;
}
