use crate::taxonomy::removal_types_vo::RemovalOptions;

/// Inbound protocol for orchestrating background removal workflows.
/// This defines the formal boundary that all inbound use cases must implement.
///
/// # Implementations
/// - RemovalUseCase (in Capabilities layer)
///
/// # Safety
/// Trait requires Send and Sync constraints for safe concurrent operations.
pub trait RemovalUseCaseProtocol: Send + Sync {
    fn execute(&self, options: &RemovalOptions) -> anyhow::Result<()>;
}
