use crate::taxonomy::{EngineNameVo, ModelPathVo, TensorDataVo};

/// Outbound port for executing background removal using ONNX Runtime.
pub trait OnnxRemoverPort: Send + Sync {
    fn onnx_get_engine_name(&self) -> EngineNameVo;

    fn onnx_run_inference(
        &self,
        model_path: &ModelPathVo,
        input_tensor: &TensorDataVo,
    ) -> anyhow::Result<TensorDataVo>;
}
