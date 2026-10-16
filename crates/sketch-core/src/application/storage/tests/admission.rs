use super::*;
#[test]
fn admission_enforces_edit_and_export_capabilities_before_copying() {
    for (edit, export) in [(false, false), (false, true), (true, false)] {
        let mut session = EditorSession::new(
            Capabilities {
                edit_document: edit,
                export_document: export,
            },
            ToolId::Select,
            session().view_state().brush,
            None,
        )
        .unwrap();
        let mut queue = coordinator();
        assert_eq!(
            queue.request(&mut session, RequestId(1), SavePurpose::SaveAs, DESTINATION),
            Err(StorageError::CapabilityDenied)
        );
        let checkpoint = queue.request(
            &mut session,
            RequestId(2),
            SavePurpose::Checkpoint,
            DESTINATION,
        );
        assert_eq!(checkpoint.is_ok(), edit);
        let copy = queue.request(
            &mut session,
            RequestId(3),
            SavePurpose::ExportCopy,
            DESTINATION,
        );
        assert_eq!(copy.is_ok(), export);
    }
}
#[test]
fn bounded_slots_and_memory_reject_without_changing_session() {
    let mut session = session();
    let bytes = session.snapshot_memory().unwrap();
    let before = session.view_state();
    let mut queue = SaveCoordinator::new(StorageLimits {
        jobs: 2,
        snapshot_bytes: bytes - 1,
    })
    .unwrap();
    assert_eq!(
        queue.request(&mut session, RequestId(1), SavePurpose::SaveAs, DESTINATION),
        Err(StorageError::Busy)
    );
    assert_eq!(queue.outstanding(), 0);
    assert_eq!(session.view_state(), before);
    assert!(!session.save_state().binding_pending());
    let mut queue = SaveCoordinator::new(StorageLimits {
        jobs: 1,
        snapshot_bytes: bytes,
    })
    .unwrap();
    request(&mut queue, &mut session, SavePurpose::ExportCopy);
    assert_eq!(queue.reserved_snapshot_bytes(), bytes);
    assert_eq!(
        queue.request(
            &mut session,
            RequestId(2),
            SavePurpose::ExportCopy,
            DESTINATION
        ),
        Err(StorageError::Busy)
    );
}
#[test]
fn save_as_remains_pending_until_owner_consumes_result_and_failure_preserves_binding() {
    let mut session = bound();
    let mut queue = coordinator();
    let token = queue
        .request(
            &mut session,
            RequestId(1),
            SavePurpose::SaveAs,
            DestinationId(2),
        )
        .unwrap();
    let job = queue.start_next().unwrap();
    queue
        .finish(job.complete(WriteOutcome::Failed(StorageFailure::Access)))
        .unwrap();
    assert_eq!(
        queue.request(
            &mut session,
            RequestId(2),
            SavePurpose::SaveAs,
            DestinationId(3)
        ),
        Err(StorageError::BindingPending)
    );
    let result = queue.take_completion(token).unwrap();
    assert_eq!(session.acknowledge_save(result), Acknowledgement::Failed);
    assert_eq!(session.save_state().destination(), Some(DESTINATION));
    assert!(!session.save_state().binding_pending());
}
#[test]
fn capture_rejects_active_input_then_excludes_cancelled_predictions() {
    let mut session = session();
    let mut queue = coordinator();
    draft(&mut session);
    assert_eq!(
        queue.request(
            &mut session,
            RequestId(1),
            SavePurpose::Checkpoint,
            DESTINATION
        ),
        Err(StorageError::InvalidTransition)
    );
    assert_eq!(queue.outstanding(), 0);
    execute(&mut session, EditorCommand::CancelInteraction);
    request(&mut queue, &mut session, SavePurpose::Checkpoint);
    assert_eq!(queue.start_next().unwrap().encode().unwrap(), FIXTURE);
}
