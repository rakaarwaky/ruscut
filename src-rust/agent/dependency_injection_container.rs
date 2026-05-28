use std::sync::Arc;
use crate::contract::{RemovalUseCaseProtocol, DiContainerAggregate};
use crate::infrastructure::huggingface_model_adapter::HuggingfaceModelAdapter;
use crate::infrastructure::onnx_remover_adapter::OnnxRemoverAdapter;
use crate::infrastructure::amdgpu_remover_adapter::DirectAmdgpuRemoverAdapter;
use crate::infrastructure::ffmpeg_video_adapter::FfmpegVideoAdapter;
use crate::capabilities::removal_usecase_executor::RemovalUseCase;

/// Composition root that wires concrete adapters to the use case.
pub struct DependencyInjectionContainer {
    removal_usecase: Arc<dyn RemovalUseCaseProtocol>,
}

impl DependencyInjectionContainer {
    pub fn new() -> Self {
        let downloader = Arc::new(HuggingfaceModelAdapter::new());
        let onnx_remover = Arc::new(OnnxRemoverAdapter::new());
        let direct_remover = Arc::new(DirectAmdgpuRemoverAdapter::new());
        let video_processor = Arc::new(FfmpegVideoAdapter::new());

        let removal_usecase = Arc::new(RemovalUseCase::new(
            downloader,
            onnx_remover,
            direct_remover,
            video_processor,
        ));

        Self { removal_usecase }
    }

    /// Forward to the aggregate trait implementation.
    pub fn get_usecase(&self) -> Arc<dyn RemovalUseCaseProtocol> {
        DiContainerAggregate::get_usecase(self)
    }
}

impl DiContainerAggregate for DependencyInjectionContainer {
    /// Returns a clone of the inner `Arc<dyn RemovalUseCaseProtocol>`.
    fn get_usecase(&self) -> Arc<dyn RemovalUseCaseProtocol> {
        Arc::clone(&self.removal_usecase)
    }
}

impl Default for DependencyInjectionContainer {
    fn default() -> Self {
        Self::new()
    }
}
