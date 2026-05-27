use std::path::PathBuf;
use colored::Colorize;
use crate::agent::BgRemoverOrchestrator;
use crate::taxonomy::removal_types_vo::{get_default_output_path, ModelType, RemovalOptions};
use crate::contract::RemovalTransferAggregate;
use dialoguer::{Confirm, Input, Select};

pub struct TuiCommandHandler {
    _dummy: bool,
}

impl TuiCommandHandler {
    pub fn new() -> Self {
        Self { _dummy: true }
    }

    pub fn run(&self, orchestrator: &BgRemoverOrchestrator) -> anyhow::Result<()> {
        println!("\n{}", "=== MODE INTERAKTIF TUI RUSCUT ===".green().bold());
        println!("Tekan Enter setelah mengisi, gunakan tombol arah ↑/↓ untuk memilih.\n");

        let input_path = loop {
            let path_str: String = Input::new()
                .with_prompt("Masukkan path file gambar input (misal: gambar.jpg)")
                .interact_text()?;
            let path = PathBuf::from(path_str.trim());
            if path.exists() {
                break path;
            }
            println!(
                "{} File tidak ditemukan di path: {:?}. Silakan masukkan path yang benar.",
                "ERROR:".red().bold(),
                path
            );
        };

        let model_selection = Select::new()
            .with_prompt("Pilih tingkat akurasi & ukuran model AI")
            .default(0)
            .items(&[
                "BRIA RMBG-1.4 Quantized (44.4 MB - Rekomendasi/Sangat Cepat)",
                "BRIA RMBG-1.4 FP16 (88.2 MB - Akurat/Seimbang)",
                "BRIA RMBG-1.4 Full (176 MB - Maksimal/Paling Detail)"
            ])
            .interact()?;

        let model_type = match model_selection {
            1 => ModelType::Fp16,
            2 => ModelType::Full,
            _ => ModelType::Quantized,
        };

        let output_str: String = Input::new()
            .with_prompt("Masukkan path hasil simpan (kosongkan untuk default)")
            .allow_empty(true)
            .interact_text()?;
        let output_path = if output_str.trim().is_empty() {
            get_default_output_path(&input_path)
        } else {
            PathBuf::from(output_str.trim())
        };

        let force_download = Confirm::new()
            .with_prompt("Apakah Anda ingin memaksa unduh ulang model AI?")
            .default(false)
            .interact()?;

        println!("{}", "\nSedang mengonfigurasi pemrosesan latar belakang...".blue().bold());

        let options = RemovalOptions {
            input_path,
            output_path,
            custom_model_path: None,
            model_type,
            force_download,
        };

        // Warn if JPG is specified (doesn't support transparency)
        if let Some(ext) = options.output_path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if ext_str == "jpg" || ext_str == "jpeg" {
                println!(
                    "{} Format JPG tidak mendukung transparansi. Latar belakang yang dihapus akan berwarna hitam/putih. Disarankan menggunakan format PNG atau WebP.",
                    "PERINGATAN:".yellow().bold()
                );
            }
        }

        // Wrap in L3 Contract IO and execute
        let io = RemovalTransferAggregate::new(options);
        orchestrator.execute(&io.options)?;

        Ok(())
    }
}

impl Default for TuiCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}
