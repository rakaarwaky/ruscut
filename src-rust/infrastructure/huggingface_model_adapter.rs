use crate::contract::ModelDownloaderPort;
use crate::taxonomy::removal_types_vo::{ModelType, get_cache_dir};
use anyhow::Context;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};

/// SHA256 checksums for verified model downloads (prevent tampering).
/// Update these values after downloading the actual model files.
/// Use `sha256sum ~/.cache/ruscut/rmbg-2.0.onnx` to compute.
const MODEL_CHECKSUMS: &[(&str, &str)] =
    &[("rmbg-2.0.onnx", "PLACEHOLDER_SHA256_UPDATE_AFTER_DOWNLOAD")];

/// Adapter that downloads ONNX models from HuggingFace Hub silently.
pub struct HuggingfaceModelAdapter {
    enabled: bool,
}

impl HuggingfaceModelAdapter {
    pub fn new() -> Self {
        Self { enabled: true }
    }
}

impl Default for HuggingfaceModelAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelDownloaderPort for HuggingfaceModelAdapter {
    fn downloader_ensure_model(
        &self,
        model_type: &ModelType,
        force: bool,
    ) -> anyhow::Result<PathBuf> {
        if !self.enabled {
            anyhow::bail!("Huggingface model adapter is disabled");
        }
        let cache_dir = get_cache_dir();
        let target_path = cache_dir.join(model_type.filename());

        if force || !target_path.exists() {
            download_model_impl(model_type.url(), &target_path, model_type.filename())?;
        }

        // Verify model integrity after download or if it already exists
        let should_verify = MODEL_CHECKSUMS
            .iter()
            .find(|(name, _)| *name == model_type.filename())
            .map(|(_, sha)| *sha)
            .filter(|sha| *sha != "PLACEHOLDER_SHA256_UPDATE_AFTER_DOWNLOAD");

        if let Some(expected_sha) = should_verify
            && !verify_checksum(&target_path, expected_sha)?
        {
            let _ = std::fs::remove_file(&target_path);
            anyhow::bail!(
                "Model integrity check failed for '{}'! Expected SHA256: {}\n\
                 This could indicate network corruption or tampering. \
                 Re-run with --force to re-download.",
                model_type.filename(),
                expected_sha
            );
        }

        Ok(target_path)
    }
}

/// Verifies the SHA256 checksum of a file against an expected value.
fn verify_checksum(path: &Path, expected: &str) -> anyhow::Result<bool> {
    let mut file =
        std::fs::File::open(path).context("Failed to open model file for checksum verification")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .context("Failed to read model chunk for verification")?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let actual = format!("{:x}", hasher.finalize());
    Ok(actual == expected)
}

/// Download buffer size (16 KB).
const DOWNLOAD_BUFFER_SIZE: usize = 16384;

fn download_model_impl(url: &str, dest_path: &Path, model_label: &str) -> anyhow::Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};
    use std::io::{Read, Write};

    let client = reqwest::blocking::Client::builder()
        .build()
        .context("Failed to create HTTP client")?;

    let mut request = client.get(url);
    if let Ok(token) = std::env::var("HF_TOKEN")
        && !token.is_empty()
    {
        request = request.header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token));
    }

    let mut response = request
        .send()
        .context("Failed to execute HTTP request to download model")?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED
        || response.status() == reqwest::StatusCode::FORBIDDEN
    {
        anyhow::bail!(
            "Access denied to model. This model ({}) requires authentication.\n\
             Please set the HF_TOKEN environment variable with your Hugging Face token:\n\
             $ export HF_TOKEN=hf_your_token_here\n\
             Get a token at: https://huggingface.co/settings/tokens",
            model_label
        );
    }

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to download model: HTTP status {}",
            response.status()
        );
    }

    let total_bytes = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());

    let pb = if let Some(total) = total_bytes {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::with_template(
                "  ⬇️  {msg}\n  [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA {eta})"
            )
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        bar.set_message(format!("Downloading {}...", model_label));
        bar
    } else {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("  ⬇️  {msg} {spinner} {bytes}")
                .unwrap()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
        );
        bar.set_message(format!("Downloading {}...", model_label));
        bar.enable_steady_tick(std::time::Duration::from_millis(120));
        bar
    };

    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create cache directory")?;
    }

    let mut file =
        std::fs::File::create(dest_path).context("Failed to create model file in cache")?;
    let mut buffer = [0; DOWNLOAD_BUFFER_SIZE];
    let mut downloaded: u64 = 0;
    loop {
        let bytes_read = response
            .read(&mut buffer)
            .context("Failed to read download chunk data")?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])
            .context("Failed to write model data to disk")?;
        downloaded += bytes_read as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message(format!("✅ {} downloaded.", model_label));
    Ok(())
}
