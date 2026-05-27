use std::path::{Path, PathBuf};
use anyhow::Context;
use colored::Colorize;
use crate::contract::ModelDownloaderPort;
use crate::taxonomy::removal_types_vo::{get_cache_dir, ModelType};

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
    fn ensure_model(&self, model_type: &ModelType, force: bool) -> anyhow::Result<PathBuf> {
        if !self.enabled {
            anyhow::bail!("Huggingface model adapter is disabled");
        }
        let cache_dir = get_cache_dir();
        let target_path = cache_dir.join(model_type.filename());

        if force || !target_path.exists() {
            download_model_impl(model_type.url(), &target_path, model_type.label())?;
        } else {
            println!(
                "{} Using model from cache: {:?}",
                "INFO:".blue().bold(),
                target_path
            );
        }

        Ok(target_path)
    }
}

fn download_model_impl(url: &str, dest_path: &Path, model_label: &str) -> anyhow::Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};
    use std::io::{Read, Write};

    println!(
        "{} Model {} not found in cache.",
        "INFO:".blue().bold(),
        model_label.cyan()
    );
    println!("{} Downloading model from Hugging Face...", "INFO:".blue().bold());

    let mut response = reqwest::blocking::get(url)
        .context("Failed to execute HTTP request to download model")?;
        
    if !response.status().is_success() {
        anyhow::bail!("Failed to download model: HTTP status {}", response.status());
    }

    let total_size = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|ct_len| ct_len.to_str().ok())
        .and_then(|ct_len| ct_len.parse::<u64>().ok())
        .unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .context("Invalid progress bar template format")?
            .progress_chars("#>-"),
    );

    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create cache directory")?;
    }

    let mut file = std::fs::File::create(dest_path).context("Failed to create model file in cache")?;
    let mut buffer = [0; 16384]; // 16KB buffer
    loop {
        let bytes_read = response
            .read(&mut buffer)
            .context("Failed to read download chunk data")?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])
            .context("Failed to write model data to disk")?;
        pb.inc(bytes_read as u64);
    }

    pb.finish_with_message("Download complete!");
    println!("{} Model saved to: {:?}", "SUCCESS:".green().bold(), dest_path);
    Ok(())
}
