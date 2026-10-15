use super::*;
#[test]
fn newer_save_of_old_destination_does_not_supersede_save_as_in_either_ack_order() {
    for reverse in [false, true] {
        let mut session = bound();
        let mut queue = coordinator();
        queue
            .request(
                &mut session,
                RequestId(1),
                SavePurpose::SaveAs,
                DestinationId(2),
            )
            .unwrap();
        stroke(&mut session);
        request(&mut queue, &mut session, SavePurpose::Save);
        let save_as = written(&mut queue);
        let save_old = written(&mut queue);
        if reverse {
            assert_eq!(
                session.acknowledge_save(save_old),
                Acknowledgement::SavedCurrent
            );
            assert_eq!(
                session.acknowledge_save(save_as),
                Acknowledgement::SavedOlder
            );
        } else {
            assert_eq!(
                session.acknowledge_save(save_as),
                Acknowledgement::SavedOlder
            );
            assert_eq!(
                session.acknowledge_save(save_old),
                Acknowledgement::StaleBinding
            );
        }
        assert_eq!(session.save_state().destination(), Some(DestinationId(2)));
        assert!(session.view_state().document.dirty);
        execute(&mut session, EditorCommand::Undo);
        assert!(!session.view_state().document.dirty);
    }
}
#[test]
fn same_destination_save_as_keeps_newer_written_revision_when_acks_reverse() {
    let mut session = bound();
    let mut queue = coordinator();
    let binding = session.save_state().binding_generation();
    request(&mut queue, &mut session, SavePurpose::SaveAs);
    stroke(&mut session);
    request(&mut queue, &mut session, SavePurpose::Save);
    let save_as = written(&mut queue);
    let newer = written(&mut queue);
    assert_eq!(
        session.acknowledge_save(newer),
        Acknowledgement::SavedCurrent
    );
    assert_eq!(
        session.acknowledge_save(save_as),
        Acknowledgement::SavedCurrent
    );
    assert_eq!(session.save_state().binding_generation(), binding + 1);
    assert!(!session.view_state().document.dirty);
    execute(&mut session, EditorCommand::Undo);
    assert!(session.view_state().document.dirty);
}
#[test]
fn uncertain_active_write_never_assumes_prior_file_or_clean_state_survived() {
    let mut session = bound();
    let mut queue = coordinator();
    let token = request(&mut queue, &mut session, SavePurpose::Save);
    let job = queue.start_next().unwrap();
    queue
        .finish(job.complete(WriteOutcome::Uncertain(StorageFailure::WorkerStopped)))
        .unwrap();
    let completion = queue.take_completion(token).unwrap();
    assert_eq!(
        completion.result(),
        SaveResult::Uncertain(StorageFailure::WorkerStopped)
    );
    assert_eq!(
        session.acknowledge_save(completion),
        Acknowledgement::Failed
    );
    assert!(session.view_state().document.dirty);
}
#[test]
fn pending_save_as_destination_rejects_copy_and_checkpoint_until_acknowledged() {
    let mut session = bound();
    let mut queue = coordinator();
    queue
        .request(
            &mut session,
            RequestId(1),
            SavePurpose::SaveAs,
            DestinationId(2),
        )
        .unwrap();
    stroke(&mut session);
    for purpose in [SavePurpose::ExportCopy, SavePurpose::Checkpoint] {
        assert_eq!(
            queue.request(&mut session, RequestId(2), purpose, DestinationId(2)),
            Err(StorageError::BindingPending)
        );
    }
    session.acknowledge_save(written(&mut queue));
    assert!(
        queue
            .request(
                &mut session,
                RequestId(3),
                SavePurpose::ExportCopy,
                DestinationId(2)
            )
            .is_ok()
    );
}
