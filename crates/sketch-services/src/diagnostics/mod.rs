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
    NativeStarted = 9,
    NativeCompleted = 10,
    NativeFailed = 11,
    ExportDispatched = 12,
}
impl EventCode {
    // Keep this exhaustive: a new variant requires an explicit counter slot.
    pub(super) const fn counter(self) -> usize {
        match self {
            Self::SaveStarted => 0,
            Self::SaveWritten => 1,
            Self::SaveFailed => 2,
            Self::SaveUncertain => 3,
            Self::CleanupFailed => 4,
            Self::RecoveryReadFailed => 5,
            Self::CheckpointPublished => 6,
            Self::RecoveryCapacity => 7,
            Self::SinkFailed => 8,
            Self::NativeStarted => 9,
            Self::NativeCompleted => 10,
            Self::NativeFailed => 11,
            Self::ExportDispatched => 12,
        }
    }
    pub(super) const COUNTERS: usize = Self::ExportDispatched.counter() + 1;
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
    pub operation_id: u64,
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
            "{{\"v\":1,\"build\":{},\"operation_id\":{},\"session\":{},\"request\":{},\"revision\":{},\"subsystem\":{},\"code\":{},\"duration_us\":{},\"count\":{},\"failure\":{},\"stage\":{},\"durability\":{},\"cleanup_unconfirmed\":{}}}\n",
            self.build,
            self.operation_id,
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
