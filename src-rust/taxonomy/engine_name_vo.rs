/// Value Object representing a validated engine name identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineNameVo {
    name: &'static str,
}

impl EngineNameVo {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }

    pub fn as_str(&self) -> &'static str {
        self.name
    }
}

impl std::fmt::Display for EngineNameVo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
