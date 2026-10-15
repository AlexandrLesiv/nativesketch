mod ring;
mod rotation;
pub use ring::{DiagnosticRing, DiagnosticStats};
pub use rotation::RotatingSink;
/// Stable numeric codes; no arbitrary strings, document content, paths or samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventCode {
    SaveStarted,
    SaveWritten,
    SaveFailed,
    SaveUncertain,
    CleanupFailed,
    RecoveryReadFailed,
    CheckpointPublished,
    RecoveryCapacity,
    SinkFailed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Subsystem {
    Storage,
    Recovery,
    Diagnostics,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event {
    pub build: u64,
    pub session: u64,
    pub request: u64,
    pub revision: u64,
    pub subsystem: Subsystem,
    pub code: EventCode,
    pub duration_us: u64,
    pub count: u64,
    pub failure: Option<sketch_core::application::storage::StorageFailure>,
    pub stage: Option<crate::storage::Stage>,
    pub durability: Option<sketch_core::application::storage::Durability>,
    pub cleanup_unconfirmed: bool,
}
impl Event {
    pub fn encode(self) -> String {
        format!(
            "{{\"v\":1,\"build\":{},\"session\":{},\"request\":{},\"revision\":{},\"subsystem\":{},\"code\":{},\"duration_us\":{},\"count\":{},\"failure\":{},\"stage\":{},\"durability\":{},\"cleanup_unconfirmed\":{}}}\n",
            self.build,
            self.session,
            self.request,
            self.revision,
            self.subsystem as u8,
            self.code as u8,
            self.duration_us,
            self.count,
            self.failure.map_or(0, |value| value as u8 + 1),
            self.stage.map_or(0, |value| value as u8 + 1),
            self.durability.map_or(0, |value| value as u8 + 1),
            self.cleanup_unconfirmed
        )
    }
}
