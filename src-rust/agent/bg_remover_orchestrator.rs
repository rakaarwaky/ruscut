use std::sync::Arc;
use crate::contract::{RemovalUseCaseProtocol, BgRemoverAggregate};
use crate::taxonomy::{BenchmarkReportVo, EngineNameVo, RemovalOptions};

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
        BgRemoverAggregate::aggregate_execute(self, options)
    }
}

impl RemovalUseCaseProtocol for BgRemoverOrchestrator {
    fn usecase_execute(&self, options: &RemovalOptions) -> anyhow::Result<()> {
        self.usecase.usecase_execute(options)
    }

    fn usecase_run_benchmark(&self) -> anyhow::Result<BenchmarkReportVo> {
        self.usecase.usecase_run_benchmark()
    }

    fn usecase_get_engine_name(&self) -> EngineNameVo {
        self.usecase.usecase_get_engine_name()
    }
}

impl BgRemoverAggregate for BgRemoverOrchestrator {
    fn aggregate_execute(&self, options: &RemovalOptions) -> anyhow::Result<()> {
        self.usecase.usecase_execute(options)
    }

    fn usecase(&self) -> &dyn RemovalUseCaseProtocol {
        self.usecase.as_ref()
    }
}
