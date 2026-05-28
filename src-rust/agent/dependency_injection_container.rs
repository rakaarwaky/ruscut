use crate::capabilities::image_data_processor::ImageDataProcessor;
use crate::capabilities::removal_usecase_executor::RemovalUseCase;
use crate::contract::{DiContainerAggregate, RemovalUseCaseProtocol};
use crate::infrastructure::amdgpu_remover_adapter::DirectAmdgpuRemoverAdapter;
use crate::infrastructure::ffmpeg_video_adapter::FfmpegVideoAdapter;
use crate::infrastructure::huggingface_model_adapter::HuggingfaceModelAdapter;
use crate::infrastructure::onnx_remover_adapter::OnnxRemoverAdapter;
use crate::infrastructure::vulkan_compute_provider::VulkanComputeEngine;
use crate::taxonomy::app_config_vo::AppConfig;
use std::sync::Arc;

/// Composition root that wires concrete adapters to the use case.
pub struct DependencyInjectionContainer {
    removal_usecase: Arc<dyn RemovalUseCaseProtocol>,
    config: AppConfig,
}

impl DependencyInjectionContainer {
    pub fn new() -> anyhow::Result<Self> {
        let config = AppConfig::load().unwrap_or_else(|e| {
            eprintln!("Failed to load config: {}. Using defaults.", e);
            AppConfig::default()
        });

        let downloader = Arc::new(HuggingfaceModelAdapter::new());
        let onnx_remover = Arc::new(OnnxRemoverAdapter::new());

        // We forbid CPU-only, so we MUST successfully initialize VulkanComputeEngine.
        let engine = VulkanComputeEngine::new().map_err(|e| {
            anyhow::anyhow!(
                "Vulkan GPU engine is not running/available: {}. CPU-only execution is forbidden.",
                e
            )
        })?;

        tracing::info!("Vulkan GPU engine initialized successfully");
        let direct_remover = Arc::new(DirectAmdgpuRemoverAdapter::with_engine(Arc::new(engine)));
        let gpu_available = true;

        let video_processor = Arc::new(FfmpegVideoAdapter::new());
        let image_processor = Arc::new(ImageDataProcessor::new());

        let removal_usecase = Arc::new(RemovalUseCase::new_with_gpu(
            downloader,
            onnx_remover,
            direct_remover,
            video_processor,
            image_processor,
            gpu_available,
        ));

        Ok(Self {
            removal_usecase,
            config,
        })
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
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
