use std::path::PathBuf;
use std::sync::Arc;
use colored::Colorize;
use crate::agent::BgRemoverOrchestrator;
use crate::contract::BgRemoverAggregate;
use crate::taxonomy::removal_types_vo::{get_cache_dir, get_default_output_path, ModelType, RemovalOptions};
use crate::taxonomy::RemovalTransferVo;
use dialoguer::{Confirm, Input, Select};

#[derive(Default)]
pub struct TuiCommandHandler {
    _initialized: bool,
}

impl TuiCommandHandler {
    pub fn new() -> Self {
        Self { _initialized: false }
    }

    pub async fn run(&self, orchestrator: &BgRemoverOrchestrator) -> anyhow::Result<()> {
        loop {
            println!("\n{}", "╔══════════════════════════════════════════════════════════╗".blue().bold());
            println!("║  {}                                                    ║", "____                 _   ".cyan().bold());
            println!("║ {}                                                   ║", " |  _ \\ _   _ ___  ___| |_ ".cyan().bold());
            println!("║ {}                                                   ║", " | |_) | | | / __|/ __| __|".cyan().bold());
            println!("║ {}                                                   ║", " |  _ <| |_| \\__ \\ (__| |_ ".cyan().bold());
            println!("║ {}                                                   ║", " |_| \\_\\\\__,_|___/\\___|\\__|".cyan().bold());
            println!("║                                                          ║");
            println!("║       {}             ║", "AI-Powered Background Remover - v0.1.0".white().bold());
            println!("{}", "╚══════════════════════════════════════════════════════════╝".blue().bold());

            let menu_items = &[
                "📷 Remove Background (Start Wizard)",
                "⚙  Settings & Configuration",
                "🩺 System Diagnostics & Health Check",
                "ℹ  About Ruscut",
                "🚪 Exit Application"
            ];

            let selection = Select::new()
                .with_prompt("Select an option from the main menu")
                .default(0)
                .items(menu_items)
                .interact()?;

            match selection {
                0 => {
                    if let Err(err) = self.start_wizard(orchestrator).await {
                        println!("\n{} Execution error: {:?}", "ERROR:".red().bold(), err);
                    }
                }
                1 => {
                    self.show_settings();
                }
                2 => {
                    self.show_diagnostics();
                }
                3 => {
                    self.show_about();
                }
                _ => {
                    println!("\n{}", "Thank you for using Ruscut! Have a wonderful day! 👋".green().bold());
                    break;
                }
            }
        }

        Ok(())
    }

    async fn start_wizard(&self, orchestrator: &BgRemoverOrchestrator) -> anyhow::Result<()> {
        println!("\n{}", "--- BACKGROUND REMOVAL WIZARD ---".green().bold());

        let input_path = loop {
            let path_str: String = Input::new()
                .with_prompt("Enter input image path (e.g. image.jpg)")
                .interact_text()?;
            let path = PathBuf::from(path_str.trim());
            if path.exists() {
                break path;
            }
            println!(
                "{} File not found at path: {:?}. Please enter a valid path.",
                "ERROR:".red().bold(),
                path
            );
        };

        let output_str: String = Input::new()
            .with_prompt("Enter output save path (leave empty for default)")
            .allow_empty(true)
            .interact_text()?;
        let output_path = if output_str.trim().is_empty() {
            get_default_output_path(&input_path)
        } else {
            PathBuf::from(output_str.trim())
        };

        let force_download = Confirm::new()
            .with_prompt("Do you want to force re-download the AI model?")
            .default(false)
            .interact()?;

        println!("{}", "\nConfiguring background processing...".blue().bold());

        let options = RemovalOptions {
            input_path,
            output_path,
            custom_model_path: None,
            model_type: ModelType::Full,
            force_download,
        };

        if let Some(ext) = options.output_path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if ext_str == "jpg" || ext_str == "jpeg" {
                println!(
                    "{} JPG format does not support transparency. The removed background will appear as solid color. PNG or WebP is recommended.",
                    "WARNING:".yellow().bold()
                );
            }
        }

        let arc_orch: Arc<BgRemoverOrchestrator> = Arc::new(orchestrator.clone());
        let io = RemovalTransferVo::new(options);
        let opts = io.options;

        tokio::task::spawn_blocking(move || {
            BgRemoverAggregate::execute(&*arc_orch, &opts)
        }).await
            .map_err(|e| anyhow::anyhow!("Task join error: {:?}", e))?
            .map_err(|e| anyhow::anyhow!("Background removal failed: {:?}", e))?;

        println!("\nPress Enter to return to the main menu...");
        let _: String = Input::new().allow_empty(true).interact_text()?;
        Ok(())
    }

    fn show_settings(&self) {
        println!("\n{}", "--- SETTINGS & CONFIGURATIONS ---".magenta().bold());
        println!("  - Model Engine  : ONNX Runtime");
        println!("  - Active Model  : BRIA RMBG-2.0 (1.02 GB)");
        println!("  - Cache Path    : {:?}", get_cache_dir());
        println!("  - Precision     : f32 (Full Precision)");

        println!("\nPress Enter to return to the main menu...");
        let _ = Input::<String>::new().allow_empty(true).interact_text();
    }

    fn show_diagnostics(&self) {
        println!("\n{}", "--- SYSTEM DIAGNOSTICS ---".yellow().bold());

        let cache_dir = get_cache_dir();
        let model_path = cache_dir.join(ModelType::Full.filename());

        println!("  - Cache Directory : {:?}", cache_dir);
        if cache_dir.exists() {
            println!("    - Directory status: {}", "ACCESSIBLE (OK)".green());
        } else {
            println!("    - Directory status: {}", "NOT CREATED YET".yellow());
        }

        println!("  - Gated AI Model  : {:?}", model_path);
        if model_path.exists() {
            if let Ok(metadata) = std::fs::metadata(&model_path) {
                let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
                println!("    - Model status    : {} ({:.2} MB)", "CACHED".green(), size_mb);
            } else {
                println!("    - Model status    : {}", "CACHED (Unable to read size)".green());
            }
        } else {
            println!("    - Model status    : {}", "MISSING (Will download automatically on execution)".yellow());
        }

        println!("  - System Target   : {}", std::env::consts::OS.to_uppercase());
        println!("  - CPU Architecture: {}", std::env::consts::ARCH.to_uppercase());

        println!("\nPress Enter to return to the main menu...");
        let _ = Input::<String>::new().allow_empty(true).interact_text();
    }

    fn show_about(&self) {
        println!("\n{}", "--- ABOUT RUSCUT ---".cyan().bold());
        println!("  An ultra high-performance local background removal tool built in Rust");
        println!("  using strict AES (Architecture Enforcement System) Clean Architecture.");
        println!();
        println!("  - Version      : 0.1.0");
        println!("  - License      : MIT License");
        println!("  - Repository   : https://github.com/rakaarwaky/ruscut");
        println!("  - Core AI Model: BRIA RMBG-2.0 (ONNX Edition)");
        println!("  - Under Gated  : Subject to Bria AI Commercial guidelines");

        println!("\nPress Enter to return to the main menu...");
        let _ = Input::<String>::new().allow_empty(true).interact_text();
    }
}
