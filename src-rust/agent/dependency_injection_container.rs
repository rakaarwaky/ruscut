use std::sync::Arc;
use crate::contract::RemovalUseCaseProtocol;
use crate::infrastructure::huggingface_model_adapter::HuggingfaceModelAdapter;
use crate::infrastructure::onnx_remover_adapter::OnnxRemoverAdapter;
use crate::capabilities::removal_usecase_executor::RemovalUseCase;
use crate::taxonomy::removal_types_vo::RemovalOptions;

pub struct DependencyInjectionContainer {
    removal_usecase: Arc<dyn RemovalUseCaseProtocol>,
}

impl DependencyInjectionContainer {
    pub fn new() -> Self {
        // 1. Instantiate concrete adapters (L4 Infrastructure)
        let downloader = Arc::new(HuggingfaceModelAdapter::new());
        let remover = Arc::new(OnnxRemoverAdapter::new());

        // 2. Instantiate and wire up the capability (L3 Capabilities) with DI ports
        let removal_usecase = Arc::new(RemovalUseCase::new(downloader, remover));

        Self { removal_usecase }
    }

    pub fn get_usecase(&self) -> Arc<dyn RemovalUseCaseProtocol> {
        let _ = Option::<RemovalOptions>::None;
        Arc::clone(&self.removal_usecase)
    }
}

impl Default for DependencyInjectionContainer {
    fn default() -> Self {
        Self::new()
    }
}
