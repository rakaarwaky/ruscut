use super::tui_state_store::{AppStateStore, JobStatus};
use crate::agent::BgRemoverOrchestrator;
use crate::contract::{BgRemoverAggregate, RemovalUseCaseProtocol};
use crate::taxonomy::removal_types_vo::{ModelType, get_cache_dir};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph},
};

pub struct TuiViewController;

impl TuiViewController {
    pub fn render_ui(
        frame: &mut Frame,
        state: &mut AppStateStore,
        orchestrator: &BgRemoverOrchestrator,
    ) {
        let area = frame.area();

        // 1. Divide layout vertically into: Header, Main Content, and Footer
        let layout_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Header with Logo
                Constraint::Fill(1),   // Main Dashboard body
                Constraint::Length(3), // Interactive Footer
            ])
            .split(area);

        Self::draw_header(frame, layout_chunks[0]);
        Self::draw_dashboard(frame, state, layout_chunks[1], orchestrator);
        Self::draw_footer(frame, layout_chunks[2]);
    }

    fn draw_header(frame: &mut Frame, area: Rect) {
        let logo = "  ____                             _   
 |  _ \\  _   _  ___   ___  _   _  | |_ 
 | |_) || | | |/ __| / __|| | | | | __|
 |  _ < | |_| |\\__ \\| (__ | |_| | | |_ 
 |_| \\_\\ \\__,_||___/ \\___| \\__,_|  \\__|";

        let header_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let logo_widget = Paragraph::new(logo).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(logo_widget, header_layout[0]);

        let title_widget = Paragraph::new(format!(
            "\nAI-Powered Background Remover - v{}\nClean architecture & local execution.",
            env!("CARGO_PKG_VERSION")
        ))
        .alignment(ratatui::layout::Alignment::Right)
        .style(Style::default().fg(Color::DarkGray).italic());
        frame.render_widget(title_widget, header_layout[1]);
    }

    fn draw_dashboard(
        frame: &mut Frame,
        state: &mut AppStateStore,
        area: Rect,
        orchestrator: &BgRemoverOrchestrator,
    ) {
        // Divide dashboard horizontally into Left (File Explorer) and Right (Details & Status)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(45), // File explorer
                Constraint::Percentage(55), // Progress & System diagnostics
            ])
            .split(area);

        Self::draw_file_explorer(frame, state, main_chunks[0]);
        Self::draw_details_panel(frame, state, main_chunks[1], orchestrator);
    }

    fn draw_file_explorer(frame: &mut Frame, state: &mut AppStateStore, area: Rect) {
        let items: Vec<ListItem> = state
            .items
            .iter()
            .map(|(display_name, _, is_dir)| {
                let style = if *is_dir {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(display_name.clone()).style(style)
            })
            .collect();

        let dir_title = format!(
            " 📁 Directory Explorer (Path: {}) ",
            state.current_dir.to_string_lossy()
        );
        let list_widget = List::new(items)
            .block(
                Block::default()
                    .title(dir_title.cyan().bold())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" > ");

        frame.render_stateful_widget(list_widget, area, &mut state.list_state);
    }

    fn draw_details_panel(
        frame: &mut Frame,
        state: &mut AppStateStore,
        area: Rect,
        orchestrator: &BgRemoverOrchestrator,
    ) {
        // Divide the right panel vertically into: Diagnostics Panel and Active Job Status
        let panel_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Diagnostics & Info
                Constraint::Percentage(50), // Active background removal job
            ])
            .split(area);

        Self::draw_diagnostics(frame, state, panel_chunks[0], orchestrator);
        Self::draw_job_status(frame, state, panel_chunks[1]);
    }

    fn draw_diagnostics(
        frame: &mut Frame,
        state: &AppStateStore,
        area: Rect,
        orchestrator: &BgRemoverOrchestrator,
    ) {
        let cache_dir = get_cache_dir();
        let model_path = cache_dir.join(ModelType::Full.filename());
        let is_cached = model_path.exists();

        let cache_status = if is_cached {
            if let Ok(metadata) = std::fs::metadata(&model_path) {
                let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
                format!("CACHED ({:.2} MB)", size_mb).green().bold()
            } else {
                "CACHED (Size unknown)".green().bold()
            }
        } else {
            "MISSING (Will auto-download)".yellow().bold()
        };

        let force_status = if state.force_download {
            "ENABLED (Will redownload AI model)".red().bold()
        } else {
            "DISABLED".dark_gray()
        };

        // Query the orchestrator container to confirm model readiness via aggregate trait method
        let engine_ready = if BgRemoverAggregate::is_ready(orchestrator).is_ok() {
            "ACTIVE (OK)".green().bold()
        } else {
            "UNAVAILABLE".red().bold()
        };

        let info_text = format!(
            "  - Model Variant   : {}
  - AI Engine       : {} [{}]
  - Platform/OS     : {} ({})
  - Cache Location  : {:?}
  - Model Status    : {}
  - Force Download  : {} [Press 'f' to toggle]",
            ModelType::Full.label().cyan(),
            orchestrator
                .usecase_get_engine_name()
                .as_str()
                .cyan()
                .bold(),
            engine_ready,
            std::env::consts::OS.to_uppercase(),
            std::env::consts::ARCH.to_uppercase(),
            cache_dir,
            cache_status,
            force_status
        );

        let diagnostics_widget = Paragraph::new(info_text).block(
            Block::default()
                .title(" 🩺 System Diagnostics & Health ".blue().bold())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue)),
        );

        frame.render_widget(diagnostics_widget, area);
    }

    fn draw_job_status(frame: &mut Frame, state: &AppStateStore, area: Rect) {
        // Build state-based prompt messages
        let mut status_message = String::new();

        let text_color = match &state.job_status {
            JobStatus::Idle => {
                status_message.push_str("Status: IDLE\nSelect an image or video file in the explorer and press [Enter] to start.");
                Color::DarkGray
            }
            JobStatus::Starting => {
                status_message.push_str(
                    "Status: STARTING WIZARD...\nInitializing async threads and locking models.",
                );
                Color::Blue
            }
            JobStatus::LoadingModel => {
                status_message.push_str("Status: LOADING AI MODEL...\nBRIA RMBG-2.0 is loading into ONNX runtime memory (this can take 3-10s depending on your CPU).");
                Color::Yellow
            }
            JobStatus::RemovingBackground => {
                status_message.push_str("Status: EXECUTING INFERENCE...\nApplying neural network to remove background pixels.");
                Color::Cyan
            }
            JobStatus::SavingOutput => {
                status_message
                    .push_str("Status: SAVING FILE...\nWriting transparent outputs onto disk.");
                Color::Magenta
            }
            JobStatus::Success(out_path) => {
                status_message.push_str(&format!(
                    "Status: SUCCESS! 🎉\nRemoved background saved successfully to:\n{:?}",
                    out_path
                ));
                Color::Green
            }
            JobStatus::Failed(err) => {
                status_message.push_str(&format!("Status: FAILED ❌\nExecution error:\n{}", err));
                Color::Red
            }
        };

        // Add file detail info if a file is being processed
        let mut detailed_text = String::new();
        if let Some(ref input) = state.active_input {
            detailed_text.push_str(&format!(
                "  - Processing File  : {}\n  - Target Output    : {}\n\n",
                input
                    .file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default()
                    .yellow(),
                state
                    .active_output
                    .as_ref()
                    .map(|o| o.to_string_lossy())
                    .unwrap_or_default()
                    .cyan()
            ));
        }
        detailed_text.push_str(&status_message);

        // Sub-layout for drawing the text logs and the progress bar gauge
        let job_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),   // Text description
                Constraint::Length(3), // Progress Gauge
            ])
            .split(area);

        let status_block = Block::default()
            .title(" ⚙️ Background Removal Engine ".blue().bold())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue));

        let detailed_widget = Paragraph::new(detailed_text)
            .style(Style::default().fg(text_color))
            .block(status_block);
        frame.render_widget(detailed_widget, job_chunks[0]);

        // Draw progress gauge
        let gauge_widget = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(
                Style::default()
                    .fg(if state.progress == 100 {
                        Color::Green
                    } else {
                        Color::Cyan
                    })
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .percent(state.progress);
        frame.render_widget(gauge_widget, job_chunks[1]);
    }

    fn draw_footer(frame: &mut Frame, area: Rect) {
        let footer_text = "  ▲/▼ : Navigate Explorer  |  Enter : Select Folder / Execute Process  |  f : Toggle Force Model Download
    Backspace/Esc : Go Up One Directory Folder  |  q : Quit Application and Restore Terminal";

        let footer_widget = Paragraph::new(footer_text)
            .alignment(ratatui::layout::Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );

        frame.render_widget(footer_widget, area);
    }
}
