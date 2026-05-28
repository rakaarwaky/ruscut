use std::sync::Arc;
use crate::taxonomy::removal_types_vo::RemovalOptions;
use crate::contract::RemovalUseCaseProtocol;

/// Aggregate contract for the dependency injection container.
///
/// The DI container acts as the composition root that wires together
/// all infrastructure adapters, capability executors, and agent-level
/// orchestrators to satisfy the removal use case protocol.
///
/// Implementors must produce a fully constructed use case with all
/// downstream dependencies (model downloader, ONNX remover) injected.
pub trait DiContainerAggregate: Send + Sync {
    /// Retrieve the fully-wired removal use case protocol implementation.
    fn get_usecase(&self) -> Arc<dyn RemovalUseCaseProtocol>;

    /// Check whether the container can handle removal for the given options.
    fn can_handle(&self, _options: &RemovalOptions) -> bool {
        true
    }
}
