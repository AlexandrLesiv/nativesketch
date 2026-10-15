mod envelope;
mod inventory;
mod management;
mod resolution;
mod retention;
pub use management::DiscardProgress;
pub use retention::RetentionReport;
mod listing;
mod schedule;
mod store;
pub use envelope::{CheckpointMetadata, RecoveryEnvelope, RecoveryId};
pub use listing::{RecoveryListing, RecoveryUsage};
pub use schedule::{CheckpointSchedule, ScheduledCheckpoint};
use std::io;
pub use store::{RecoveryCandidate, RecoveryStore};
#[derive(Debug, Clone, Copy)]
pub struct RecoveryLimits {
    pub sessions: usize,
    pub bytes: u64,
}
impl Default for RecoveryLimits {
    fn default() -> Self {
        Self {
            sessions: 20,
            bytes: 256 * 1024 * 1024,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryError {
    Invalid,
    UnsafeEntry,
    Capacity,
    Conflict,
    Io,
    Missing,
}
impl From<io::Error> for RecoveryError {
    fn from(_: io::Error) -> Self {
        Self::Io
    }
}
pub(crate) const MAX_ENVELOPE: u64 = 64 * 1024 * 1024 + 1024;
pub(crate) const RETENTION_SECONDS: u64 = 7 * 24 * 60 * 60;
mod empty;
