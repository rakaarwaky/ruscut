use std::sync::Arc;
use crate::contract::{BackgroundRemoverPort, ModelDownloaderPort, RemovalUseCaseProtocol};
use crate::taxonomy::removal_types_vo::RemovalOptions;

pub struct RemovalUseCase {
    downloader: Arc<dyn ModelDownloaderPort>,
    remover: Arc<dyn BackgroundRemoverPort>,
}

impl RemovalUseCase {
    pub fn new(
        downloader: Arc<dyn ModelDownloaderPort>,
        remover: Arc<dyn BackgroundRemoverPort>,
    ) -> Self {
        Self { downloader, remover }
    }
}

impl RemovalUseCaseProtocol for RemovalUseCase {
    fn execute(&self, options: &RemovalOptions) -> anyhow::Result<()> {
        // 1. Determine the ONNX model path
        let model_path = if let Some(ref custom_path) = options.custom_model_path {
            if !custom_path.exists() {
                anyhow::bail!("File model custom tidak ditemukan di path: {:?}", custom_path);
            }
            custom_path.clone()
        } else {
            self.downloader.ensure_model(&options.model_type, options.force_download)?
        };

        // 2. Process image using the remover port
        self.remover.remove_background(&model_path, &options.input_path, &options.output_path, &options.model_type)?;

        Ok(())
    }
}
