use crate::taxonomy::removal_types_vo::RemovalOptions;

/// Value Object (VO) that wraps background removal request data.
/// Follows strict three-word naming and mandatory _vo taxonomy suffix rules.
pub struct RemovalTransferVo {
    pub options: RemovalOptions,
}

impl RemovalTransferVo {
    pub fn new(options: RemovalOptions) -> Self {
        Self { options }
    }
}
