use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum ModelType {
    Quantized,
    Fp16,
    Full,
}

impl ModelType {
    pub fn url(&self) -> &'static str {
        match self {
            ModelType::Quantized => "https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model_quantized.onnx",
            ModelType::Fp16 => "https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model_fp16.onnx",
            ModelType::Full => "https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model.onnx",
        }
    }

    pub fn filename(&self) -> &'static str {
        match self {
            ModelType::Quantized => "rmbg-1.4-quantized.onnx",
            ModelType::Fp16 => "rmbg-1.4-fp16.onnx",
            ModelType::Full => "rmbg-1.4-full.onnx",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ModelType::Quantized => "BRIA RMBG-1.4 Quantized (44.4 MB)",
            ModelType::Fp16 => "BRIA RMBG-1.4 FP16 (88.2 MB)",
            ModelType::Full => "BRIA RMBG-1.4 Full (176 MB)",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemovalOptions {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub custom_model_path: Option<PathBuf>,
    pub model_type: ModelType,
    pub force_download: bool,
}

pub fn get_cache_dir() -> PathBuf {
    if let Some(mut path) = dirs::cache_dir() {
        path.push("ruscut");
        path
    } else {
        PathBuf::from(".cache")
    }
}

pub fn get_default_output_path(input_path: &Path) -> PathBuf {
    let mut output_path = input_path.to_path_buf();
    let file_stem = input_path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| std::borrow::Cow::Borrowed("output"));
    let new_filename = format!("{}_no_bg.png", file_stem);
    output_path.set_file_name(new_filename);
    output_path
}
