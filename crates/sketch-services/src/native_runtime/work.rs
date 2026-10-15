use crate::recovery::{CheckpointMetadata, RecoveryError, RecoveryId, RecoveryListing};
use crate::worker::ServiceReceipt;
use sketch_core::application::EditorSession;
use sketch_core::application::routing::{
    ApplicationSession, PreparedReplacement, ReplacementFailure, ReplacementTicket,
};
use sketch_core::application::storage::{DestinationId, WriteJob};
use std::path::PathBuf;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeFailure {
    Access,
    Capacity,
    Stopped,
    Panic,
    Unsupported,
    Corrupt,
    Stale,
    Sink,
    Recovery(RecoveryError),
}
pub enum Retired {
    Editor(EditorSession),
    Session(ApplicationSession),
    Candidate(PreparedReplacement),
}
pub enum Work {
    Resolve(PathBuf),
    Prepare {
        ticket: ReplacementTicket,
        recovery: Option<CheckpointMetadata>,
    },
    Write {
        job: WriteJob,
        checkpoint: Option<CheckpointMetadata>,
    },
    Register(RecoveryId),
    List,
    Discard(CheckpointMetadata),
    ResolveRecovery {
        metadata: CheckpointMetadata,
        now_seconds: u64,
    },
    Retry,
    Diagnostics(PathBuf),
    Retire(Box<Retired>),
    ReleaseDestination(DestinationId),
}
pub struct WorkerRequest {
    pub operation: u64,
    pub work: Work,
}
pub enum WorkerValue {
    Started {
        recovery: Result<(), NativeFailure>,
        diagnostics: Result<(), NativeFailure>,
    },
    Destination(DestinationId),
    Replacement(Box<Result<PreparedReplacement, ReplacementFailure>>),
    Write(ServiceReceipt),
    Prepared(Box<PreparedReplacement>),
    ReplacementFailed(Option<ReplacementFailure>),
    Completed(
        Option<sketch_core::application::storage::SaveCompletion>,
        crate::storage::StoreReport,
    ),
    Inventory(RecoveryListing),
    Complete,
    Failure(NativeFailure),
}
pub struct WorkerResult {
    pub operation: u64,
    pub value: WorkerValue,
    pub diagnostic_drops: u64,
    pub diagnostic_sink_failures: u64,
}
