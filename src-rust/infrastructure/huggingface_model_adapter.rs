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
                "{} Menggunakan model dari cache: {:?}",
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
        "{} Model {} tidak ditemukan di cache.",
        "INFO:".blue().bold(),
        model_label.cyan()
    );
    println!("{} Mengunduh model dari Hugging Face...", "INFO:".blue().bold());

    let mut response = reqwest::blocking::get(url)
        .context("Gagal melakukan HTTP request untuk mengunduh model")?;
        
    if !response.status().is_success() {
        anyhow::bail!("Gagal mengunduh model: status HTTP {}", response.status());
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
            .context("Format progress bar tidak valid")?
            .progress_chars("#>-"),
    );

    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).context("Gagal membuat direktori cache")?;
    }

    let mut file = std::fs::File::create(dest_path).context("Gagal membuat file model di cache")?;
    let mut buffer = [0; 16384]; // 16KB buffer
    loop {
        let bytes_read = response
            .read(&mut buffer)
            .context("Gagal membaca chunk data unduhan")?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])
            .context("Gagal menulis data model ke disk")?;
        pb.inc(bytes_read as u64);
    }

    pb.finish_with_message("Unduhan selesai!");
    println!("{} Model disimpan ke: {:?}", "SUKSES:".green().bold(), dest_path);
    Ok(())
}
