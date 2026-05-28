//! Ruscut TUI binary — interactive menu-driven background remover.

pub mod agent;
pub mod capabilities;
pub mod contract;
pub mod infrastructure;
pub mod surfaces;
pub mod taxonomy;

use crate::surfaces::tui_command_page::TuiCommandPage;
use crate::taxonomy::app_config_vo::AppConfig;
use colored::Colorize;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing(config: &AppConfig) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("ruscut={}", config.app.log_level)));

    let cache_dir = if let Some(mut path) = dirs::cache_dir() {
        path.push("ruscut");
        path
    } else {
        std::path::PathBuf::from(".cache")
    };

    let _ = std::fs::create_dir_all(&cache_dir);
    let log_file_path = cache_dir.join("ruscut.log");

    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
    {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(file)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();
    }
}

#[tokio::main]
async fn main() {
    let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!(
            "{} Failed to load config: {}. Using defaults.",
            "WARNING:".yellow(),
            e
        );
        AppConfig::default()
    });

    init_tracing(&config);

    if config.app.safe_mode || std::env::var("RUSCUT_SAFE_MODE").is_ok() {
        crate::infrastructure::pci_bar_provider::disable_unsafe_operations();
    }

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "Ruscut TUI starting");

    let container = match agent::DependencyInjectionContainer::new() {
        Ok(c) => c,
        Err(err) => {
            tracing::error!(error = %err, "Initialization failed");
            eprintln!("{} {:?}", "ERROR:".red().bold(), err);
            std::process::exit(1);
        }
    };
    let orchestrator =
        std::sync::Arc::new(agent::BgRemoverOrchestrator::new(container.get_usecase()));

    let handler = TuiCommandPage::new();
    if let Err(err) = handler.run(&orchestrator).await {
        tracing::error!(error = %err, "TUI execution failed");
        eprintln!("{} {:?}", "ERROR:".red().bold(), err);
        std::process::exit(1);
    }

    tracing::info!("Ruscut TUI exited successfully");
}
