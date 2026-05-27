use std::path::Path;
use crate::taxonomy::removal_types_vo::ModelType;

/// Outbound port for executing background removal inference.
pub trait BackgroundRemoverPort: Send + Sync {
    fn remove_background(
        &self,
        model_path: &Path,
        input_path: &Path,
        output_path: &Path,
        model_type: &ModelType,
    ) -> anyhow::Result<()>;
}
