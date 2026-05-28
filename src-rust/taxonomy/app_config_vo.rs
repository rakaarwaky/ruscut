use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app: AppSettings,
    pub models: ModelSettings,
    pub inference: InferenceSettings,
    pub output: OutputSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub cache_dir: Option<PathBuf>,
    pub log_level: String,
    pub enable_telemetry: bool,
    pub safe_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    pub rmbg_2_0: ModelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub url: String,
    pub sha256: String,
    pub cache_ttl_hours: u64,
    pub fallback_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceSettings {
    pub prefer_gpu: bool,
    pub gpu_vendor_filter: Option<String>,
    pub max_concurrent: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSettings {
    pub default_format: String,
    pub compression_level: u8,
    pub preserve_metadata: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSettings {
                cache_dir: None,
                log_level: "info".to_string(),
                enable_telemetry: false,
                safe_mode: false,
            },
            models: ModelSettings {
                rmbg_2_0: ModelConfig {
                    url: "https://huggingface.co/yuvraj108c/RMBG-2.0/resolve/main/onnx/model.onnx"
                        .to_string(),
                    sha256: "PLACEHOLDER_SHA256_UPDATE_AFTER_DOWNLOAD".to_string(),
                    cache_ttl_hours: 168,
                    fallback_url: None,
                },
            },
            inference: InferenceSettings {
                prefer_gpu: true,
                gpu_vendor_filter: None,
                max_concurrent: 4,
                timeout_seconds: 300,
            },
            output: OutputSettings {
                default_format: "png".to_string(),
                compression_level: 6,
                preserve_metadata: true,
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let mut builder = config::Config::builder();

        // Try to load config/default.toml
        if std::path::Path::new("config/default.toml").exists() {
            builder = builder.add_source(config::File::with_name("config/default").required(false));
        }

        // Try to load config/local.toml (user overrides)
        if std::path::Path::new("config/local.toml").exists() {
            builder = builder.add_source(
                config::File::with_name("config/local")
                    .required(false)
                    .format(config::FileFormat::Toml),
            );
        }

        // Load environment variables with RUSCUT prefix
        builder = builder.add_source(config::Environment::with_prefix("RUSCUT").separator("__"));

        let config = builder.build()?;
        let config: AppConfig = config.try_deserialize()?;
        Ok(config)
    }

    pub fn get_cache_dir(&self) -> PathBuf {
        self.app
            .cache_dir
            .clone()
            .unwrap_or_else(crate::taxonomy::removal_types_vo::get_cache_dir)
    }
}
