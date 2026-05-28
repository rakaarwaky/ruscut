use std::sync::Arc;
use crate::contract::{RemovalUseCaseProtocol, BgRemoverAggregate};
use crate::taxonomy::removal_types_vo::RemovalOptions;

#[derive(Clone)]
pub struct BgRemoverOrchestrator {
    usecase: Arc<dyn RemovalUseCaseProtocol>,
}

impl BgRemoverOrchestrator {
    pub fn new(usecase: Arc<dyn RemovalUseCaseProtocol>) -> Self {
        Self { usecase }
    }

    /// Forward to the aggregate trait implementation.
    pub fn execute(&self, options: &RemovalOptions) -> anyhow::Result<()> {
        BgRemoverAggregate::execute(self, options)
    }
}

impl RemovalUseCaseProtocol for BgRemoverOrchestrator {
    fn execute(&self, options: &RemovalOptions) -> anyhow::Result<()> {
        self.usecase.execute(options)
    }
}

impl BgRemoverAggregate for BgRemoverOrchestrator {
    fn execute(&self, options: &RemovalOptions) -> anyhow::Result<()> {
        self.usecase.execute(options)
    }

    fn usecase(&self) -> &dyn RemovalUseCaseProtocol {
        self.usecase.as_ref()
    }
}
