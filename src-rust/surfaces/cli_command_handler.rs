use std::path::PathBuf;
use clap::Parser;
use colored::Colorize;
use crate::agent::BgRemoverOrchestrator;
use crate::taxonomy::removal_types_vo::{get_default_output_path, ModelType, RemovalOptions};
use crate::contract::RemovalTransferAggregate;

#[derive(Parser, Debug)]
#[command(name = "ruscut")]
#[command(author = "Antigravity <google-deepmind>")]
#[command(version = "0.1.0")]
#[command(about = "AI-powered Background Remover CLI in Rust (AES Architecture)", long_about = None)]
struct Args {
    /// Path to input image file (e.g. JPG, PNG, WebP)
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Path to output transparent PNG image file. Defaults to <input>_no_bg.png
    #[arg(value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Path to a custom .onnx model file instead of auto-downloading BRIA RMBG-1.4
    #[arg(short, long, value_name = "MODEL_PATH")]
    model: Option<PathBuf>,

    /// Force re-download the model even if it exists in cache
    #[arg(short, long)]
    force_download: bool,
}

pub struct CliCommandHandler {
    _dummy: bool,
}

impl CliCommandHandler {
    pub fn new() -> Self {
        Self { _dummy: true }
    }

    pub fn run(&self, orchestrator: &BgRemoverOrchestrator) -> anyhow::Result<()> {
        let args = Args::parse();

        // 1. Validate input file
        if !args.input.exists() {
            anyhow::bail!("Input image file not found at path: {:?}", args.input);
        }

        // 2. Determine output file
        let output_path = args.output.clone().unwrap_or_else(|| get_default_output_path(&args.input));

        // 3. Warn if JPG is specified (doesn't support transparency)
        if let Some(ext) = output_path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if ext_str == "jpg" || ext_str == "jpeg" {
                println!(
                    "{} JPG format does not support transparency. The removed background will appear as solid color. PNG or WebP is recommended.",
                    "WARNING:".yellow().bold()
                );
            }
        }

        // 4. Model type is always Full (single model architecture)
        let model_type = ModelType::Full;

        // 5. Map arguments to L1 Taxonomy value object
        let options = RemovalOptions {
            input_path: args.input,
            output_path,
            custom_model_path: args.model,
            model_type,
            force_download: args.force_download,
        };

        // 6. Wrap in L3 Contract aggregate and execute
        let io = RemovalTransferAggregate::new(options);
        orchestrator.execute(&io.options)?;

        Ok(())
    }
}

impl Default for CliCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}
