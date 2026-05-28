use std::path::{Path, PathBuf};

/// Supported AI model variants for background removal.
#[derive(Debug, Clone)]
pub enum ModelType {
    /// Full-size BRIA RMBG-2.0 model (1024x1024 input, ~1.02 GB).
    Full,
}

impl ModelType {
    /// HuggingFace download URL for this model variant.
    pub fn url(&self) -> &'static str {
        "https://huggingface.co/yuvraj108c/RMBG-2.0/resolve/main/onnx/model.onnx"
    }

    /// Cache filename for this model variant.
    pub fn filename(&self) -> &'static str {
        "rmbg-2.0.onnx"
    }

    /// Human-readable label for this model variant.
    pub fn label(&self) -> &'static str {
        "BRIA RMBG-2.0 (1.02 GB)"
    }
}

/// Options for a single background removal operation.
#[derive(Debug, Clone)]
pub struct RemovalOptions {
    /// Path to the input image file.
    pub input_path: PathBuf,
    /// Path where the output image will be saved.
    pub output_path: PathBuf,
    /// Optional path to a custom ONNX model file (bypasses auto-download).
    pub custom_model_path: Option<PathBuf>,
    /// Which model variant to use.
    pub model_type: ModelType,
    /// If true, re-download the model even if it already exists in cache.
    pub force_download: bool,
}

/// Returns the platform-specific cache directory for Ruscut models.
pub fn get_cache_dir() -> PathBuf {
    if let Some(mut path) = dirs::cache_dir() {
        path.push("ruscut");
        path
    } else {
        PathBuf::from(".cache")
    }
}

/// Generates a default output path by appending `_no_bg.png` to the input filename.
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
