use sketch_core::application::storage::{Durability, StorageFailure};
use sketch_services::diagnostics::{Event, EventCode, Subsystem};
use sketch_services::storage::Stage;
#[test]
fn diagnostic_schema_v1_numeric_codes_and_absent_fields_are_frozen() {
    assert_eq!(
        [
            EventCode::SaveStarted,
            EventCode::SaveWritten,
            EventCode::SaveFailed,
            EventCode::SaveUncertain,
            EventCode::CleanupFailed,
            EventCode::RecoveryReadFailed,
            EventCode::CheckpointPublished,
            EventCode::RecoveryCapacity,
            EventCode::SinkFailed,
            EventCode::NativeStarted,
            EventCode::NativeCompleted,
            EventCode::NativeFailed
        ]
        .map(|code| code as u8),
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
    );
    assert_eq!(
        [
            Subsystem::Storage,
            Subsystem::Recovery,
            Subsystem::Diagnostics
        ]
        .map(|s| s as u8),
        [0, 1, 2]
    );
    assert_eq!(
        [
            Stage::BeforeCreate,
            Stage::AfterCreate,
            Stage::AfterWrite,
            Stage::AfterFileSync,
            Stage::BeforeReplace,
            Stage::AfterReplace,
            Stage::BeforeDirectorySync,
            Stage::BeforeCleanup
        ]
        .map(|stage| stage as u8 + 1),
        [1, 2, 3, 4, 5, 6, 7, 8]
    );
    assert_eq!(
        [
            StorageFailure::Encode,
            StorageFailure::Access,
            StorageFailure::Capacity,
            StorageFailure::Write,
            StorageFailure::Replace,
            StorageFailure::WorkerStopped
        ]
        .map(|failure| failure as u8 + 1),
        [1, 2, 3, 4, 5, 6]
    );
    assert_eq!(
        [
            Durability::FileAndDirectorySynced,
            Durability::DirectorySyncUnconfirmed,
            Durability::ProviderConfirmed
        ]
        .map(|durability| durability as u8 + 1),
        [1, 2, 3]
    );
    let event = Event {
        operation_id: 0,
        build: 1,
        session: 2,
        request: 3,
        revision: 4,
        subsystem: Subsystem::Recovery,
        code: EventCode::CheckpointPublished,
        duration_us: 5,
        count: 6,
        failure: None,
        stage: None,
        durability: None,
        cleanup_unconfirmed: false,
    };
    assert_eq!(
        event.encode(),
        concat!(
            "{\"v\":1,\"build\":1,\"operation_id\":0,\"session\":2,\"request\":3,\"revision\":4,",
            "\"subsystem\":1,\"code\":6,\"duration_us\":5,\"count\":6,",
            "\"failure\":0,\"stage\":0,\"durability\":0,\"cleanup_unconfirmed\":false}\n"
        )
    );
}
