use super::super::{DocumentSessionId, RequestId, Revision};

/// Host-assigned canonical destination identity. Never contains a path or access grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DestinationId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavePurpose {
    Save,
    SaveAs,
    Checkpoint,
    ExportCopy,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageLimits {
    pub jobs: usize,
    pub snapshot_bytes: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    Busy,
    CapabilityDenied,
    InvalidBinding,
    BindingPending,
    ResourceLimit,
    UnknownToken,
    InvalidTransition,
}
/// Correlation only; possession does not authorize an unissued completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveToken {
    pub(crate) session: DocumentSessionId,
    pub(crate) request: RequestId,
    pub(crate) revision: Revision,
    pub(crate) destination: DestinationId,
    pub(crate) binding: u64,
    pub(crate) sequence: u64,
    pub(crate) purpose: SavePurpose,
}
impl SaveToken {
    pub fn session(self) -> DocumentSessionId {
        self.session
    }
    pub fn request(self) -> RequestId {
        self.request
    }
    pub fn revision(self) -> Revision {
        self.revision
    }
    pub fn destination(self) -> DestinationId {
        self.destination
    }
    pub fn binding_generation(self) -> u64 {
        self.binding
    }
    pub fn sequence(self) -> u64 {
        self.sequence
    }
    pub fn purpose(self) -> SavePurpose {
        self.purpose
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    FileAndDirectorySynced,
    DirectorySyncUnconfirmed,
    ProviderConfirmed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageFailure {
    Encode,
    Access,
    Capacity,
    Write,
    Replace,
    WorkerStopped,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteOutcome {
    /// ExportCopy dispatch only; not confirmation of provider persistence.
    DispatchedUnconfirmed,
    Written(Durability),
    /// The adapter cannot establish whether replacement happened; never assume unchanged.
    Uncertain(StorageFailure),
    /// The adapter guarantees that destination replacement did not happen.
    Failed(StorageFailure),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveResult {
    DispatchedUnconfirmed,
    DispatchedAfterCancellation,
    Written(Durability),
    Uncertain(StorageFailure),
    Failed(StorageFailure),
    CancelledBeforeStart,
    WrittenAfterCancellation(Durability),
    FailedAfterCancellation(StorageFailure),
    UncertainAfterCancellation(StorageFailure),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acknowledgement {
    SavedCurrent,
    SavedOlder,
    CopyWritten,
    CopyDispatched,
    CheckpointWritten,
    StaleSession,
    StaleBinding,
    Superseded,
    Cancelled,
    Failed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SaveState {
    pub(crate) destination: Option<DestinationId>,
    pub(crate) binding: u64,
    pub(crate) last_ack: u64,
    pub(crate) pending_binding: Option<u64>,
    pub(crate) pending_destination: Option<DestinationId>,
}
impl SaveState {
    pub fn destination(self) -> Option<DestinationId> {
        self.destination
    }
    pub fn binding_generation(self) -> u64 {
        self.binding
    }
    pub fn binding_pending(self) -> bool {
        self.pending_binding.is_some()
    }
}
