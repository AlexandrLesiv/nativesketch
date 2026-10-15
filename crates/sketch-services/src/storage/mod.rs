mod destination;
mod staged;
pub use destination::DestinationAccess;
pub(crate) use destination::validate_directory;
use sketch_core::application::storage::WriteOutcome;
pub use staged::write;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    BeforeCreate,
    AfterCreate,
    AfterWrite,
    AfterFileSync,
    BeforeReplace,
    AfterReplace,
    BeforeDirectorySync,
    BeforeCleanup,
}
/// Fault/process-kill hooks receive no content or private path.
pub trait StoreHooks: Send + Sync {
    fn at(&self, stage: Stage) -> io::Result<()>;
}
pub struct NoFaults;
impl StoreHooks for NoFaults {
    fn at(&self, _: Stage) -> io::Result<()> {
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreReport {
    pub outcome: WriteOutcome,
    pub cleanup_failed: bool,
    pub directory_sync_failed: bool,
    pub failed_stage: Option<Stage>,
}
