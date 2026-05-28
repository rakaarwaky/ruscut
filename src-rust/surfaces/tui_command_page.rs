use crate::agent::BgRemoverOrchestrator;
use crate::contract::BgRemoverAggregate;
use crate::taxonomy::removal_types_vo::{ModelType, RemovalOptions, get_default_output_path};
use std::io;
use std::panic;

use super::tui_state_store::{AppStateStore, JobStatus};
use super::tui_view_controller::TuiViewController;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

pub struct TuiCommandPage {
    _initialized: bool,
}

impl TuiCommandPage {
    pub fn new() -> Self {
        Self {
            _initialized: false,
        }
    }

    pub async fn run(&self, orchestrator: &BgRemoverOrchestrator) -> anyhow::Result<()> {
        // Setup raw terminal mode and enter alternate buffer
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Set up panic hook to automatically restore terminal raw mode if the TUI crashes
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            let mut stdout = io::stdout();
            let _ = execute!(stdout, LeaveAlternateScreen);
            let _ = disable_raw_mode();
            original_hook(panic_info);
        }));

        let mut app_state = AppStateStore::new();
        let (tx, rx) = std::sync::mpsc::channel::<JobStatus>();

        loop {
            // Check for background thread execution updates
            while let Ok(new_status) = rx.try_recv() {
                match &new_status {
                    JobStatus::Starting => {
                        app_state.job_status = JobStatus::Starting;
                        app_state.progress = 5;
                    }
                    JobStatus::LoadingModel => {
                        app_state.job_status = JobStatus::LoadingModel;
                        app_state.progress = 35;
                    }
                    JobStatus::RemovingBackground => {
                        app_state.job_status = JobStatus::RemovingBackground;
                        app_state.progress = 70;
                    }
                    JobStatus::SavingOutput => {
                        app_state.job_status = JobStatus::SavingOutput;
                        app_state.progress = 90;
                    }
                    JobStatus::Success(out_path) => {
                        app_state.job_status = JobStatus::Success(out_path.clone());
                        app_state.progress = 100;
                    }
                    JobStatus::Failed(err) => {
                        app_state.job_status = JobStatus::Failed(err.clone());
                        app_state.progress = 0;
                    }
                    _ => {}
                }
            }

            // Draw the dashboard layout passing orchestrator reference for diagnostics
            terminal.draw(|f| TuiViewController::render_ui(f, &mut app_state, orchestrator))?;

            // Read events in a non-blocking poll to keep rendering gauge animations
            if event::poll(std::time::Duration::from_millis(50))? {
                let ev = event::read()?;
                if let Event::Key(key) = ev {
                    // Ignore key releases to prevent double actions
                    if key.kind != KeyEventKind::Release {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Up => {
                                if let Some(selected) =
                                    app_state.list_state.selected().filter(|&s| s > 0)
                                {
                                    app_state.list_state.select(Some(selected - 1));
                                }
                            }
                            KeyCode::Down => {
                                if let Some(selected) = app_state
                                    .list_state
                                    .selected()
                                    .filter(|&s| s + 1 < app_state.items.len())
                                {
                                    app_state.list_state.select(Some(selected + 1));
                                }
                            }
                            KeyCode::Esc | KeyCode::Backspace => {
                                app_state.go_up_directory();
                            }
                            KeyCode::Char('f') => {
                                app_state.force_download = !app_state.force_download;
                            }
                            KeyCode::Enter => {
                                if let Some(selected) = app_state
                                    .list_state
                                    .selected()
                                    .filter(|&s| s < app_state.items.len())
                                {
                                    let (_, path, is_dir) = &app_state.items[selected];
                                    if *is_dir {
                                        app_state.current_dir = path.clone();
                                        app_state.reload_files();
                                    } else if app_state.job_status == JobStatus::Idle
                                        || matches!(
                                            app_state.job_status,
                                            JobStatus::Success(_) | JobStatus::Failed(_)
                                        )
                                    {
                                        // Start async background removal job using the clean contract API
                                        let orch = orchestrator.clone();
                                        let input_path = path.clone();
                                        let output_path = get_default_output_path(&input_path);
                                        let force = app_state.force_download;
                                        let job_tx = tx.clone();

                                        app_state.active_input = Some(input_path.clone());
                                        app_state.active_output = Some(output_path.clone());
                                        app_state.job_status = JobStatus::Starting;
                                        app_state.progress = 5;

                                        std::thread::spawn(move || {
                                            let _ = job_tx.send(JobStatus::LoadingModel);

                                            let options = RemovalOptions {
                                                input_path,
                                                output_path,
                                                custom_model_path: None,
                                                model_type: ModelType::Full,
                                                force_download: force,
                                            };

                                            let _ = job_tx.send(JobStatus::RemovingBackground);
                                            match BgRemoverAggregate::aggregate_execute(
                                                &orch, &options,
                                            ) {
                                                Ok(_) => {
                                                    let _ = job_tx.send(JobStatus::SavingOutput);
                                                    let _ = job_tx.send(JobStatus::Success(
                                                        options.output_path,
                                                    ));
                                                }
                                                Err(e) => {
                                                    let _ = job_tx
                                                        .send(JobStatus::Failed(e.to_string()));
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Gracefully clean up raw terminal and exit alternate screen buffer
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }
}

impl Default for TuiCommandPage {
    fn default() -> Self {
        Self::new()
    }
}
