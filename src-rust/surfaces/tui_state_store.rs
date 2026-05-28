use std::path::PathBuf;
use ratatui::widgets::ListState;

#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Idle,
    Starting,
    LoadingModel,
    RemovingBackground,
    SavingOutput,
    Success(PathBuf),
    Failed(String),
}

pub struct AppStateStore {
    pub current_dir: PathBuf,
    pub items: Vec<(String, PathBuf, bool)>, // (display_name, absolute_path, is_directory)
    pub list_state: ListState,
    pub job_status: JobStatus,
    pub progress: u16,
    pub force_download: bool,
    pub active_input: Option<PathBuf>,
    pub active_output: Option<PathBuf>,
}

impl AppStateStore {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut state = Self {
            current_dir,
            items: Vec::new(),
            list_state: ListState::default(),
            job_status: JobStatus::Idle,
            progress: 0,
            force_download: false,
            active_input: None,
            active_output: None,
        };
        state.reload_files();
        state
    }

    pub fn reload_files(&mut self) {
        self.items = Vec::new();
        
        // Add parent directory navigation if it exists
        if let Some(parent) = self.current_dir.parent() {
            self.items.push(("📁 ..".to_string(), parent.to_path_buf(), true));
        }

        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();

            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                
                if path.is_dir() {
                    dirs.push((format!("📁 {}", name), path, true));
                } else {
                    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                    if ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "webp" {
                        files.push((format!("📷 {}", name), path, false));
                    }
                }
            }

            // Sort directories and files alphabetically
            dirs.sort_by(|a, b| a.0.cmp(&b.0));
            files.sort_by(|a, b| a.0.cmp(&b.0));

            self.items.extend(dirs);
            self.items.extend(files);
        }

        if self.items.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    pub fn go_up_directory(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.reload_files();
        }
    }
}

impl Default for AppStateStore {
    fn default() -> Self {
        Self::new()
    }
}
