use crate::taxonomy::engine_name_vo::EngineNameVo;
use image::DynamicImage;

/// Protocol for image preprocessing and postprocessing operations.
pub trait ImageProcessorProtocol: Send + Sync {
    /// Preprocesses image into normalized tensor for model inference.
    fn processor_preprocess(&self, image: &DynamicImage) -> anyhow::Result<Vec<f32>>;
    /// Postprocesses raw model output into grayscale mask.
    fn processor_postprocess(&self, model_result: &[f32]) -> anyhow::Result<DynamicImage>;
    /// Applies alpha mask to original image.
    fn processor_apply_mask(&self, original: &DynamicImage, mask: &DynamicImage) -> DynamicImage;
    /// Resizes image to target dimensions.
    fn processor_resize(
        &self,
        image: &DynamicImage,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Vec<u8>>;
    /// Returns the engine name for this processor.
    fn processor_engine_name(&self) -> EngineNameVo;
}
