use super::*;
#[test]
fn ordinary_save_uses_old_binding_while_save_as_is_queued() {
    let mut app = app(true, true);
    let mut queue = queue();
    save_as(&mut app, &mut queue, DestinationId(1));
    let completion = complete(&mut queue, written());
    run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut Some(completion)),
    );
    save_as(&mut app, &mut queue, DestinationId(2));
    let ApplicationEffect::SaveQueued(save) = file(&mut app, &mut queue, FileIntent::Save) else {
        panic!()
    };
    assert_eq!(save.destination(), DestinationId(1));
    let new_binding = complete(&mut queue, written());
    let old_binding = complete(&mut queue, written());
    run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut Some(old_binding)),
    );
    run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut Some(new_binding)),
    );
    assert_eq!(app.projection().save.destination(), Some(DestinationId(2)));
}
#[test]
fn late_save_during_read_blocks_install_until_trusted_completion_is_applied() {
    let mut app = app(true, true);
    let mut queue = queue();
    save_as(&mut app, &mut queue, DestinationId(1));
    let completion = complete(&mut queue, written());
    run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut Some(completion)),
    );
    let mut candidate = prepared(&mut app, &mut queue, DestinationId(2));
    file(&mut app, &mut queue, FileIntent::Save);
    let completion = complete(&mut queue, written());
    assert_eq!(queue.outstanding(), 0);
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::Install(&mut candidate)
        )
        .is_err()
    );
    assert!(!candidate.is_consumed());
    run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut Some(completion)),
    );
    run(
        &mut app,
        &mut queue,
        ApplicationCommand::Install(&mut candidate),
    );
    assert_eq!(app.projection().save.destination(), Some(DestinationId(2)));
}
#[test]
fn checkpoint_does_not_block_replacement_or_clean_recovered_state() {
    let mut app = app(true, true);
    let mut queue = queue();
    file(
        &mut app,
        &mut queue,
        FileIntent::Checkpoint(DestinationId(8)),
    );
    let ApplicationEffect::PrepareReplacement(ticket) = file(
        &mut app,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::Recover(RecoverySourceId(9))),
    ) else {
        panic!()
    };
    let mut candidate = ticket
        .prepare(Some(ValidatedDocument::decode(FIXTURE).unwrap()))
        .unwrap();
    run(
        &mut app,
        &mut queue,
        ApplicationCommand::Install(&mut candidate),
    );
    let completion = complete(&mut queue, written());
    let outcome = run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut Some(completion)),
    );
    assert_eq!(
        outcome.save.map(|save| save.acknowledgement),
        Some(Acknowledgement::StaleSession)
    );
    assert!(app.projection().editor.document.dirty);
}
#[test]
fn accepted_save_admission_failure_is_not_reported_as_rejected_draft_mutation() {
    let mut app = app(true, true);
    let mut queue = SaveCoordinator::new(StorageLimits {
        jobs: 1,
        snapshot_bytes: 1,
    })
    .unwrap();
    draft(&mut app, &mut queue);
    let ApplicationEffect::ChooseDestination { token, .. } =
        file(&mut app, &mut queue, FileIntent::SaveAs)
    else {
        panic!()
    };
    assert_eq!(app.projection().editor.interaction, InteractionState::Idle);
    assert!(matches!(
        reply(
            &mut app,
            &mut queue,
            token,
            EffectDecision::Destination(DestinationId(1))
        ),
        ApplicationEffect::Failure(ApplicationFailure::Storage(StorageError::Busy))
    ));
    assert_eq!(app.projection().save.destination(), None);
    assert_eq!(bytes(&app), FIXTURE);
}
