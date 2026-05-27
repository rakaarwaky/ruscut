pub mod model_downloader_port;
pub mod background_remover_port;
pub mod removal_usecase_protocol;
pub mod removal_transfer_aggregate;

pub use model_downloader_port::ModelDownloaderPort;
pub use background_remover_port::BackgroundRemoverPort;
pub use removal_usecase_protocol::RemovalUseCaseProtocol;
pub use removal_transfer_aggregate::RemovalTransferAggregate;

pub const BARREL: bool = true;
