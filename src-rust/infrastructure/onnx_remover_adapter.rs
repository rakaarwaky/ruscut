use std::sync::{Arc, Mutex};
use anyhow::anyhow;
use crate::contract::OnnxRemoverPort;
use crate::taxonomy::{EngineNameVo, ModelPathVo, TensorDataVo};

struct CachedSession {
    model_path: String,
    session: Arc<Mutex<ort::session::Session>>,
}

pub struct OnnxRemoverAdapter {
    enabled: bool,
    cache: Mutex<Option<CachedSession>>,
}

impl OnnxRemoverAdapter {
    pub fn new() -> Self {
        Self {
            enabled: true,
            cache: Mutex::new(None),
        }
    }

    fn load_or_reuse_session(&self, model_path: &ModelPathVo) -> anyhow::Result<Arc<Mutex<ort::session::Session>>> {
        let model_path_str = model_path.as_path().to_string_lossy().to_string();

        let mut cache = self.cache.lock()
            .expect("Failed to lock model cache mutex");
        if let Some(ref cached) = *cache && cached.model_path == model_path_str {
            return Ok(Arc::clone(&cached.session));
        }
        let mut builder = ort::session::Session::builder()?;

        let session = Arc::new(Mutex::new(builder
            .commit_from_file(model_path.as_path())
            .map_err(|e| anyhow!("Failed to load model: {:?}", e))?));

        *cache = Some(CachedSession {
            model_path: model_path_str,
            session: Arc::clone(&session),
        });

        Ok(session)
    }
}

impl Default for OnnxRemoverAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl OnnxRemoverPort for OnnxRemoverAdapter {
    fn onnx_get_engine_name(&self) -> EngineNameVo {
        EngineNameVo::new("ONNX Runtime")
    }

    fn onnx_run_inference(
        &self,
        model_path: &ModelPathVo,
        input_tensor: &TensorDataVo,
    ) -> anyhow::Result<TensorDataVo> {
        if !self.enabled {
            anyhow::bail!("ONNX remover adapter is disabled");
        }

        let model_arc = self.load_or_reuse_session(model_path)?;
        let mut model = model_arc.lock()
            .map_err(|e| anyhow!("Failed to lock model: {:?}", e))?;

        let input_name = model.inputs()[0].name().to_string();
        let output_name = model.outputs().last()
            .ok_or(anyhow!("Model has no output nodes"))?
            .name().to_string();

        let input_array = ndarray::Array4::from_shape_vec((1, 3, 1024, 1024), input_tensor.data.clone())
            .map_err(|e| anyhow!("Failed to reshape input: {:?}", e))?;

        let ort_input = ort::value::Tensor::from_array(input_array)
            .map_err(|e| anyhow!("Failed to create ORT tensor: {:?}", e))?;

        let inputs = ort::inputs![&input_name => ort_input];
        let outputs = model.run(inputs)
            .map_err(|e| anyhow!("Failed to run model: {:?}", e))?;

        let (_, slice) = outputs[output_name.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(|e| anyhow!("Failed to extract output: {:?}", e))?;

        Ok(TensorDataVo::new(slice.to_vec()))
    }
}
