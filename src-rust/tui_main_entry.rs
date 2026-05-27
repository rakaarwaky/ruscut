pub mod taxonomy;
pub mod contract;
pub mod capabilities;
pub mod infrastructure;
pub mod agent;
pub mod surfaces;

use colored::Colorize;
use crate::surfaces::tui_command_handler::TuiCommandHandler;

pub struct TuiMainEntry {
    _dummy: bool,
}

impl TuiMainEntry {
    pub fn new() -> Self {
        Self { _dummy: true }
    }
}

impl Default for TuiMainEntry {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    // 1. Initialize Dependency Injection Container (Composition Root)
    let container = agent::DependencyInjectionContainer::new();

    // 2. Resolve the stateless Orchestrator
    let orchestrator = agent::BgRemoverOrchestrator::new(container.get_usecase());

    // 3. Handover control to L6 TUI Surface Handler
    let handler = TuiCommandHandler::new();
    if let Err(err) = handler.run(&orchestrator) {
        eprintln!("{} {:?}", "ERROR:".red().bold(), err);
        std::process::exit(1);
    }
}
