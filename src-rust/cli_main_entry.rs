pub mod taxonomy;
pub mod contract;
pub mod capabilities;
pub mod infrastructure;
pub mod agent;
pub mod surfaces;

use colored::Colorize;
use crate::surfaces::cli_command_handler::CliCommandHandler;

pub struct CliMainEntry {
    _dummy: bool,
}

impl CliMainEntry {
    pub fn new() -> Self {
        Self { _dummy: true }
    }
}

impl Default for CliMainEntry {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    // 1. Initialize Dependency Injection Container (Composition Root)
    let container = agent::DependencyInjectionContainer::new();

    // 2. Resolve the stateless Orchestrator
    let orchestrator = agent::BgRemoverOrchestrator::new(container.get_usecase());

    // 3. Handover control to L6 CLI Surface Handler
    let handler = CliCommandHandler::new();
    if let Err(err) = handler.run(&orchestrator) {
        eprintln!("{} {:?}", "ERROR:".red().bold(), err);
        std::process::exit(1);
    }
}
