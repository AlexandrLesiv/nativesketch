use std::path::PathBuf;
#[derive(Clone)]
pub struct RuntimeConfig {
    pub sessions: usize,
    pub operations: usize,
    pub events: usize,
    pub snapshot_bytes: usize,
    pub build: u64,
    pub recovery_root: Option<PathBuf>,
    pub diagnostics_root: Option<PathBuf>,
}
impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            sessions: 16,
            operations: 8,
            events: 128,
            snapshot_bytes: 128 * 1024 * 1024,
            build: 0,
            recovery_root: None,
            diagnostics_root: None,
        }
    }
}
impl RuntimeConfig {
    pub fn validate(&self) -> bool {
        (1..=32).contains(&self.sessions)
            && (1..=32).contains(&self.operations)
            && self.events <= 256
            && self.events >= self.operations * 3 + self.sessions * 2 + 4
            && self.snapshot_bytes > 0
            && self.snapshot_bytes <= 512 * 1024 * 1024
            && [&self.recovery_root, &self.diagnostics_root]
                .iter()
                .all(|root| root.as_ref().is_none_or(|path| path.is_absolute()))
    }
}
