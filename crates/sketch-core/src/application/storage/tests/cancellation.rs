use super::*;
#[test]
fn queued_cancel_releases_memory_and_cannot_write_or_clean() {
    let mut session = session();
    let mut queue = coordinator();
    stroke(&mut session);
    let token = request(&mut queue, &mut session, SavePurpose::SaveAs);
    queue.cancel(token).unwrap();
    assert_eq!(queue.reserved_snapshot_bytes(), 0);
    assert!(queue.start_next().is_none());
    let result = queue.take_completion(token).unwrap();
    assert_eq!(result.result(), SaveResult::CancelledBeforeStart);
    assert_eq!(session.acknowledge_save(result), Acknowledgement::Cancelled);
    assert!(session.view_state().document.dirty);
    assert_eq!(session.save_state().destination(), None);
}
#[test]
fn cancelled_write_to_bound_file_invalidates_previously_clean_knowledge() {
    let mut session = bound();
    let mut queue = coordinator();
    stroke(&mut session);
    let token = request(&mut queue, &mut session, SavePurpose::Save);
    execute(&mut session, EditorCommand::Undo);
    assert!(!session.view_state().document.dirty);
    let job = queue.start_next().unwrap();
    queue.cancel(token).unwrap();
    assert!(queue.reserved_snapshot_bytes() > 0);
    queue
        .finish(job.complete(WriteOutcome::Written(Durability::FileAndDirectorySynced)))
        .unwrap();
    let result = queue.take_completion(token).unwrap();
    assert!(matches!(
        result.result(),
        SaveResult::WrittenAfterCancellation(_)
    ));
    assert_eq!(session.acknowledge_save(result), Acknowledgement::Cancelled);
    assert!(session.view_state().document.dirty);
    execute(&mut session, EditorCommand::Redo);
    assert!(
        session.view_state().document.dirty,
        "cancelled acknowledgement never marks clean"
    );
}
#[test]
fn cancelled_save_as_to_another_file_does_not_adopt_or_dirty_current_binding() {
    let mut session = bound();
    let mut queue = coordinator();
    let token = queue
        .request(
            &mut session,
            RequestId(2),
            SavePurpose::SaveAs,
            DestinationId(2),
        )
        .unwrap();
    let job = queue.start_next().unwrap();
    queue.cancel(token).unwrap();
    queue
        .finish(job.complete(WriteOutcome::Written(Durability::FileAndDirectorySynced)))
        .unwrap();
    session.acknowledge_save(queue.take_completion(token).unwrap());
    assert_eq!(session.save_state().destination(), Some(DESTINATION));
    assert!(!session.view_state().document.dirty);
    assert!(!session.save_state().binding_pending());
}
#[test]
fn failed_write_preserves_saved_revision_and_edits() {
    let mut session = bound();
    let mut queue = coordinator();
    stroke(&mut session);
    let token = request(&mut queue, &mut session, SavePurpose::Save);
    let job = queue.start_next().unwrap();
    queue
        .finish(job.complete(WriteOutcome::Failed(StorageFailure::Capacity)))
        .unwrap();
    assert_eq!(
        session.acknowledge_save(queue.take_completion(token).unwrap()),
        Acknowledgement::Failed
    );
    assert!(session.view_state().document.dirty);
    execute(&mut session, EditorCommand::Undo);
    assert!(!session.view_state().document.dirty);
}
