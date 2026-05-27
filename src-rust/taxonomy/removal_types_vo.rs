use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum ModelType {
    Full,
}

impl ModelType {
    pub fn url(&self) -> &'static str {
        "https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model.onnx"
    }

    pub fn filename(&self) -> &'static str {
        "rmbg-1.4-full.onnx"
    }

    pub fn label(&self) -> &'static str {
        "BRIA RMBG-1.4 Full (176 MB)"
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
