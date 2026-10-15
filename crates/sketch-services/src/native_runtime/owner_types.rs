use super::work::*;
use crate::recovery::{CheckpointMetadata, CheckpointSchedule, RecoveryId};
use sketch_core::application::{routing::*, storage::*};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeError {
    Invalid,
    Stale,
    Busy,
    Capacity,
    Owner,
    Stopped,
    Denied,
    Application(ApplicationFailure),
}
pub struct SessionSlot {
    pub window: u64,
    pub app: ApplicationSession,
    pub bound_grant: u64,
    pub close_confirmed: bool,
    pub pristine: bool,
    pub detached: bool,
    pub recovery: Option<RecoveryRegistration>,
    pub recovered_from: Option<CheckpointMetadata>,
    pub requested_recovery: Option<CheckpointMetadata>,
    pub last_checkpoint: Option<CheckpointMetadata>,
    pub schedule: CheckpointSchedule,
}
pub struct RecoveryRegistration {
    pub identity: RecoveryId,
    pub name: String,
    pub state: u32,
    pub generation: u64,
}
#[derive(Clone, Copy)]
pub enum IntentEffect {
    None,
    Destination(EffectToken, SavePurpose),
    Source(EffectToken),
    Unsaved(EffectToken),
    Writes(EffectToken),
    Save(SaveToken),
    Preparing,
    Opening,
    CloseReady(EffectToken),
    CloseConfirmed,
    Cancelled,
    Failure(ApplicationFailure),
}
pub struct ActionResult {
    pub outcome: ApplicationOutcome,
    pub effect: IntentEffect,
    pub operation: u64,
    pub retry_action: u32,
}
#[derive(Clone, Copy)]
pub enum EventKind {
    OwnerReady,
    GrantRelease,
    RecoveryReady,
    Maintenance,
    Failure,
    Shutdown,
}
#[derive(Clone, Copy)]
pub struct NativeEvent {
    pub id: u64,
    pub operation: u64,
    pub window: u64,
    pub session: u64,
    pub request: u64,
    pub grant: u64,
    pub kind: EventKind,
    pub failure: Option<NativeFailure>,
}
pub(super) enum OperationKind {
    Resolve { token: EffectToken, source: bool },
    Save(SaveToken),
    Prepare(EffectToken),
    Register,
    RetryRecovery,
    List,
    Maintenance,
    Retirement,
}
pub(super) enum OperationState {
    Queued(Box<Work>),
    WaitingWrite(SaveToken),
    Running,
    Ready(Box<WorkerValue>),
}
pub(super) struct PendingOperation {
    pub id: u64,
    pub window: u64,
    pub session: u64,
    pub request: u64,
    pub grant: u64,
    pub kind: OperationKind,
    pub state: OperationState,
    pub event: Option<u64>,
    pub retry: u32,
    pub checkpoint: Option<CheckpointMetadata>,
    pub recovery_source: Option<CheckpointMetadata>,
    pub cancelled: bool,
    pub items: Vec<(u64, CheckpointMetadata)>,
}
pub(super) struct Lease {
    pub grant: u64,
    pub destination: Option<DestinationId>,
    pub references: usize,
}
#[derive(Clone, Copy, Default)]
pub struct RuntimeState {
    pub pending: usize,
    pub ready: usize,
    pub retained: usize,
    pub attached: usize,
    pub needs_idle: bool,
    pub stopping: bool,
    pub stopped: bool,
    pub wake_after_ms: u64,
    pub diagnostic_drops: u64,
    pub diagnostic_sink_failures: u64,
}

pub(super) struct OperationOwner {
    pub window: u64,
    pub session: u64,
    pub request: u64,
    pub grant: u64,
}
pub struct RecoveryInventory<'a> {
    pub items: &'a [(u64, CheckpointMetadata)],
    pub listing: &'a crate::recovery::RecoveryListing,
}
