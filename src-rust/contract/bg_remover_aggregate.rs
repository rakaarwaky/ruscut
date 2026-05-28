use crate::taxonomy::removal_types_vo::RemovalOptions;
use crate::contract::RemovalUseCaseProtocol;

/// Aggregate contract extending `RemovalUseCaseProtocol`.
pub trait BgRemoverAggregate: RemovalUseCaseProtocol {
    /// Execute background removal with the given options.
    fn aggregate_execute(&self, options: &RemovalOptions) -> anyhow::Result<()>;

    /// Check if the aggregate is ready to process requests
    fn is_ready(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Retrieve the underlying usecase protocol reference.
    fn usecase(&self) -> &dyn RemovalUseCaseProtocol;
}
