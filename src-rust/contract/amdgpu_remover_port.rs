use crate::taxonomy::{EngineNameVo, ModelPathVo, TensorDataVo};

/// Outbound port for executing background removal directly on AMD GPU using Vulkan.
pub trait DirectAmdgpuRemoverPort: Send + Sync {
    fn amdgpu_get_engine_name(&self) -> EngineNameVo;

    fn amdgpu_run_inference(
        &self,
        model_path: &ModelPathVo,
        input_tensor: &TensorDataVo,
    ) -> anyhow::Result<TensorDataVo>;
}
