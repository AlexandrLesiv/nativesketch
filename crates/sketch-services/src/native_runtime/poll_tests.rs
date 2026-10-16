use super::{NativeRuntime, RuntimeConfig, owner_types::*};
use sketch_core::{
    Brush,
    application::{storage::*, *},
};

// SAVE-003: coordinator ownership must be probed before consuming WaitingWrite.
// This is owner-state coverage; the FFI regression exercises real file workers.
#[test]
fn blocked_or_mismatched_preflight_preserves_waiting_operation() {
    let mut runtime = NativeRuntime::new(RuntimeConfig::default()).unwrap();
    let mut editor = EditorSession::new(
        Capabilities {
            edit_document: true,
            export_document: true,
        },
        ToolId::Pen,
        Brush {
            rgba: [0.0, 0.0, 0.0, 1.0],
            width: 3.0,
        }
        .try_into()
        .unwrap(),
        None,
    )
    .unwrap();
    let first = runtime
        .coordinator
        .request(
            &mut editor,
            RequestId(1),
            SavePurpose::ExportCopy,
            DestinationId(1),
        )
        .unwrap();
    let second = runtime
        .coordinator
        .request(
            &mut editor,
            RequestId(2),
            SavePurpose::ExportCopy,
            DestinationId(2),
        )
        .unwrap();
    let operation = runtime
        .enqueue(
            OperationOwner {
                window: 0,
                session: second.session().get(),
                request: 2,
                grant: 0,
            },
            OperationKind::Save(second),
            OperationState::WaitingWrite(second),
            0,
        )
        .unwrap();
    assert_eq!(runtime.pump(), Err(RuntimeError::Invalid));
    assert!(matches!(runtime.operations[0].state,
        OperationState::WaitingWrite(token) if token == second));
    let held_job = runtime.coordinator.start_next().unwrap();
    assert_eq!(held_job.token(), first);
    assert!(runtime.coordinator.is_active(first));
    runtime.pump().unwrap();
    assert_eq!(runtime.operations[0].id, operation);
    assert!(matches!(runtime.operations[0].state,
        OperationState::WaitingWrite(token) if token == second));
    assert!(!runtime.worker_active);
    assert_eq!(runtime.coordinator.outstanding(), 2);
    // No provider was invoked in this owner-state fixture. Explicitly reject the held job.
    runtime
        .coordinator
        .finish(held_job.complete(WriteOutcome::Failed(StorageFailure::Access)))
        .unwrap();
    runtime.coordinator.take_completion(first).unwrap();
    runtime.coordinator.cancel(second).unwrap();
    runtime.coordinator.take_completion(second).unwrap();
    runtime.operations.clear();
}
