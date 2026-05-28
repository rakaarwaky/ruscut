//! Ruscut CLI binary — AI-powered background remover.

pub mod agent;
pub mod capabilities;
pub mod contract;
pub mod infrastructure;
pub mod surfaces;
pub mod taxonomy;

use crate::surfaces::cli_command_handler::CliCommandHandler;
use crate::taxonomy::app_config_vo::AppConfig;
use colored::Colorize;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing(config: &AppConfig) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("ruscut={}", config.app.log_level)));

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

fn main() {
    // Load configuration
    let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!(
            "{} Failed to load config: {}. Using defaults.",
            "WARNING:".yellow().bold(),
            e
        );
        AppConfig::default()
    });

    // Initialize tracing/logging
    init_tracing(&config);

    // Enable safe mode if configured or via environment variable
    if config.app.safe_mode || std::env::var("RUSCUT_SAFE_MODE").is_ok() {
        crate::infrastructure::pci_bar_provider::disable_unsafe_operations();
    }

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "Ruscut CLI starting");

    let container = agent::DependencyInjectionContainer::new();
    let orchestrator = agent::BgRemoverOrchestrator::new(container.get_usecase());

    let handler = CliCommandHandler::new();
    if let Err(err) = handler.run(&orchestrator) {
        tracing::error!(
            error = %err,
            "CLI execution failed"
        );
        eprintln!("{} {:?}", "ERROR:".red().bold(), err);
        std::process::exit(1);
    }

    tracing::info!("Ruscut CLI exited successfully");
}
