use super::*;
#[test]
fn fifo_jobs_hold_exact_request_revision_and_worker_cannot_overtake() {
    let mut session = bound();
    let mut queue = coordinator();
    let first = request(&mut queue, &mut session, SavePurpose::Save);
    stroke(&mut session);
    let second = request(&mut queue, &mut session, SavePurpose::Save);
    assert!(second.sequence() > first.sequence());
    assert_ne!(first.revision(), second.revision());
    let job = queue.start_next().unwrap();
    assert_eq!(job.token(), first);
    assert_eq!(job.encode().unwrap(), FIXTURE);
    assert!(queue.start_next().is_none());
    queue
        .finish(job.complete(WriteOutcome::Written(Durability::FileAndDirectorySynced)))
        .unwrap();
    let job = queue.start_next().unwrap();
    assert_eq!(job.token(), second);
    assert_ne!(job.encode().unwrap(), FIXTURE);
}
#[test]
fn unknown_duplicate_and_unfinished_completion_are_rejected() {
    let mut session = bound();
    let mut queue = coordinator();
    let token = request(&mut queue, &mut session, SavePurpose::Save);
    let mut unknown = token;
    unknown.request = RequestId(999);
    assert!(matches!(
        queue.take_completion(unknown),
        Err(StorageError::UnknownToken)
    ));
    assert!(matches!(
        queue.take_completion(token),
        Err(StorageError::InvalidTransition)
    ));
    let job = queue.start_next().unwrap();
    queue
        .finish(job.complete(WriteOutcome::Written(Durability::DirectorySyncUnconfirmed)))
        .unwrap();
    assert_eq!(
        queue
            .finish(WriteReceipt {
                token,
                outcome: WriteOutcome::Written(Durability::ProviderConfirmed)
            })
            .map_err(|error| error.error()),
        Err(StorageError::InvalidTransition)
    );
    let completion = queue.take_completion(token).unwrap();
    assert_eq!(
        completion.result(),
        SaveResult::Written(Durability::DirectorySyncUnconfirmed)
    );
    assert!(matches!(
        queue.take_completion(token),
        Err(StorageError::UnknownToken)
    ));
    assert_eq!(
        session.acknowledge_save(completion),
        Acknowledgement::SavedCurrent
    );
}
#[test]
fn one_process_queue_orders_different_windows_even_when_destination_ids_alias() {
    let mut first = bound();
    let mut second = bound();
    let mut queue = coordinator();
    let a = request(&mut queue, &mut first, SavePurpose::Save);
    let b = queue
        .request(
            &mut second,
            RequestId(2),
            SavePurpose::ExportCopy,
            DestinationId(999),
        )
        .unwrap();
    let job = queue.start_next().unwrap();
    assert_eq!(job.token(), a);
    assert!(queue.start_next().is_none());
    queue
        .finish(job.complete(WriteOutcome::Written(Durability::ProviderConfirmed)))
        .unwrap();
    assert_eq!(queue.start_next().unwrap().token(), b);
}
#[test]
fn acknowledgement_during_new_draft_preserves_active_samples() {
    let mut session = bound();
    let mut queue = coordinator();
    request(&mut queue, &mut session, SavePurpose::Save);
    draft(&mut session);
    let before = session.view_state().interaction;
    assert_eq!(
        session.acknowledge_save(written(&mut queue)),
        Acknowledgement::SavedCurrent
    );
    assert_eq!(session.view_state().interaction, before);
    execute(&mut session, EditorCommand::CommitStroke(PointerId(7)));
    assert!(session.view_state().document.dirty);
}
