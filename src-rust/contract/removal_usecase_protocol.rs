use crate::taxonomy::EngineNameVo;

/// Inbound protocol for orchestrating background removal workflows.
/// This defines the formal boundary that all inbound use cases must implement.
///
/// # Implementations
/// - RemovalUseCase (in Capabilities layer)
///
/// # Safety
/// Trait requires Send and Sync constraints for safe concurrent operations.
pub trait RemovalUseCaseProtocol: Send + Sync {
    fn usecase_execute(&self, options: &crate::taxonomy::RemovalOptions) -> anyhow::Result<()>;
    fn usecase_run_benchmark(&self) -> anyhow::Result<crate::taxonomy::BenchmarkReportVo>;
    fn usecase_get_engine_name(&self) -> EngineNameVo;
}
