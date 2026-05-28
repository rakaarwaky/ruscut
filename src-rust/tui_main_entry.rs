//! Ruscut TUI binary — interactive menu-driven background remover.

pub mod taxonomy;
pub mod contract;
pub mod capabilities;
pub mod infrastructure;
pub mod agent;
pub mod surfaces;

#[tokio::main]
async fn main() {
    let container = agent::DependencyInjectionContainer::new();
    let orchestrator = std::sync::Arc::new(agent::BgRemoverOrchestrator::new(container.get_usecase()));

    let handler = surfaces::tui_command_page::TuiCommandPage::new();
    if let Err(err) = handler.run(&orchestrator).await {
        eprintln!("ERROR: {:?}", err);
        std::process::exit(1);
    }
}
