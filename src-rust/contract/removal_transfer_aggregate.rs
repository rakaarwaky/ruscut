use crate::taxonomy::removal_types_vo::RemovalOptions;
use crate::contract::RemovalUseCaseProtocol;

pub struct RemovalTransferAggregate {
    pub options: RemovalOptions,
}

impl RemovalTransferAggregate {
    pub fn new(options: RemovalOptions) -> Self {
        Self { options }
    }

    pub fn execute_with(&self, protocol: &dyn RemovalUseCaseProtocol) -> anyhow::Result<()> {
        protocol.execute(&self.options)
    }
}
