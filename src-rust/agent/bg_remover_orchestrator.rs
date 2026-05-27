use std::sync::Arc;
use crate::contract::RemovalUseCaseProtocol;
use crate::taxonomy::removal_types_vo::RemovalOptions;

pub struct BgRemoverOrchestrator {
    usecase: Arc<dyn RemovalUseCaseProtocol>,
}

impl BgRemoverOrchestrator {
    pub fn new(usecase: Arc<dyn RemovalUseCaseProtocol>) -> Self {
        Self { usecase }
    }

    pub fn execute(&self, options: &RemovalOptions) -> anyhow::Result<()> {
        // Stateless invocation of the inbound capability protocol
        self.usecase.execute(options)
    }
}
