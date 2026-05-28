/// Value Object representing a flat float tensor for GPU inference.
#[derive(Debug, Clone)]
pub struct TensorDataVo {
    pub data: Vec<f32>,
}

impl TensorDataVo {
    pub fn new(data: Vec<f32>) -> Self {
        Self { data }
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
