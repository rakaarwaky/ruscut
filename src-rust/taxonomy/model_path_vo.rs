use std::path::PathBuf;

/// Value Object representing a validated model file path.
#[derive(Debug, Clone)]
pub struct ModelPathVo {
    pub path: PathBuf,
}

impl ModelPathVo {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn as_path(&self) -> &std::path::Path {
        &self.path
    }
}
