use std::path::PathBuf;
use clap::Parser;
use colored::Colorize;
use crate::agent::BgRemoverOrchestrator;
use crate::contract::BgRemoverAggregate;
use crate::taxonomy::removal_types_vo::{get_default_output_path, ModelType, RemovalOptions};
use crate::taxonomy::RemovalTransferVo;

#[derive(Parser, Debug)]
#[command(name = "ruscut")]
#[command(author = "RakaArwaky>")]
#[command(version = "0.1.0")]
#[command(about = "AI-powered Background Remover CLI in Rust (AES Architecture)", long_about = None)]
struct Args {
    /// Path to input image file (e.g. JPG, PNG, WebP)
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Path to output transparent PNG image file. Defaults to <input>_no_bg.png
    #[arg(value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Path to a custom .onnx model file instead of auto-downloading BRIA RMBG-2.0
    #[arg(short, long, value_name = "MODEL_PATH")]
    model: Option<PathBuf>,

    /// Force re-download the model even if it exists in cache
    #[arg(short, long)]
    force_download: bool,
}

/// CLI surface handler that parses arguments and delegates to the orchestrator.
#[derive(Default)]
pub struct CliCommandHandler {
    _initialized: bool,
}

impl CliCommandHandler {
    pub fn new() -> Self {
        Self { _initialized: false }
    }

    /// Parse CLI args via clap and execute background removal.
    pub fn run(&self, orchestrator: &BgRemoverOrchestrator) -> anyhow::Result<()> {
        let args = Args::parse();

        if !args.input.exists() {
            anyhow::bail!("Input image file not found at path: {:?}", args.input);
        }

        let output_path = args.output.clone().unwrap_or_else(|| get_default_output_path(&args.input));

        if let Some(ext) = output_path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if ext_str == "jpg" || ext_str == "jpeg" {
                println!(
                    "{} JPG format does not support transparency. The removed background will appear as solid color. PNG or WebP is recommended.",
                    "WARNING:".yellow().bold()
                );
            }
        }

        let model_type = ModelType::Full;

        let options = RemovalOptions {
            input_path: args.input,
            output_path,
            custom_model_path: args.model,
            model_type,
            force_download: args.force_download,
        };

        let io = RemovalTransferVo::new(options);
        BgRemoverAggregate::execute(orchestrator, &io.options)?;

        Ok(())
    }
}
