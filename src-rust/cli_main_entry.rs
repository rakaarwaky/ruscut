//! Ruscut CLI binary — AI-powered background remover.

pub mod taxonomy;
pub mod contract;
pub mod capabilities;
pub mod infrastructure;
pub mod agent;
pub mod surfaces;

use colored::Colorize;
use crate::surfaces::cli_command_handler::CliCommandHandler;

fn main() {
    let container = agent::DependencyInjectionContainer::new();
    let orchestrator = agent::BgRemoverOrchestrator::new(container.get_usecase());

    let handler = CliCommandHandler::new();
    if let Err(err) = handler.run(&orchestrator) {
        eprintln!("{} {:?}", "ERROR:".red().bold(), err);
        std::process::exit(1);
    }
}
