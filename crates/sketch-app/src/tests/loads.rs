use super::*;
// LOAD-001 / ASYNC-001: validation before install, busy deferral and stale retirement.
#[test]
fn corrupt_or_busy_replacement_preserves_current_view_and_valid_candidate_can_retry() {
    let (mut app, handle) = app(AppLimits::default(), true);
    let initial = app.projection(handle).unwrap().editor.session;
    let operation = open(&mut app, handle, resource(1));
    let work = read(&mut app);
    complete(&mut app, work.prepare(vec![1, 2, 3]));
    let outcome = app.apply_ready(handle, operation).unwrap();
    assert!(matches!(outcome.effect, Some(HostEffect::Failure(_))));
    assert_eq!(outcome.projection.editor.session, initial);
    app.take_release().unwrap();
    let operation = open(&mut app, handle, resource(2));
    let work = read(&mut app);
    complete(&mut app, work.prepare(FIXTURE.to_vec()));
    draft(&mut app, handle);
    assert!(matches!(
        app.apply_ready(handle, operation),
        Err(AppError::Core(ApplicationError {
            failure: ApplicationFailure::Busy,
            ..
        }))
    ));
    assert_eq!(app.projection(handle).unwrap().editor.session, initial);
    run(
        &mut app,
        handle,
        Intent::Edit(EditorCommand::CancelInteraction),
    );
    let outcome = app.apply_ready(handle, operation).unwrap();
    assert_ne!(outcome.projection.editor.session, initial);
    assert_eq!(outcome.projection.editor.document.count, 1);
    retire(&mut app);
    assert_eq!(app.stats().operations, 0);
}

#[test]
fn newer_committed_edit_rejects_prepared_load_and_retires_candidate_off_owner() {
    let (mut app, handle) = app(AppLimits::default(), true);
    let operation = open(&mut app, handle, resource(1));
    let work = read(&mut app);
    complete(&mut app, work.prepare(FIXTURE.to_vec()));
    stroke(&mut app, handle);
    let before = app.projection(handle).unwrap();
    assert!(app.apply_ready(handle, operation).is_err());
    assert_eq!(app.projection(handle).unwrap().editor, before.editor);
    assert_eq!(app.projection(handle).unwrap().file, FilePhase::Idle);
    assert_eq!(app.stats().operations, 1);
    retire(&mut app);
    assert_eq!(app.stats().operations, 0);
    assert_eq!(
        app.take_release().unwrap().descriptor.resource,
        ProviderResourceId(1)
    );
    let outcome = run(&mut app, handle, Intent::File(FileIntent::SaveAs));
    assert!(matches!(
        outcome.effect,
        Some(HostEffect::ChooseDestination { .. })
    ));
}

// VIEW-001 / SOURCE-001: newer source wins, failed latest source retains old display.
#[test]
fn viewer_source_generations_survive_replacement_and_reject_out_of_order_results() {
    let (mut app, handle) = app(AppLimits::default(), false);
    let outcome = app
        .begin_view_source(handle, ctx(&app, handle), 1, resource(1))
        .unwrap();
    let Some(HostEffect::WorkQueued(first)) = outcome.effect else {
        panic!()
    };
    let old = read(&mut app);
    let outcome = app
        .begin_view_source(handle, ctx(&app, handle), 2, resource(2))
        .unwrap();
    let Some(HostEffect::WorkQueued(second)) = outcome.effect else {
        panic!()
    };
    let latest = read(&mut app);
    complete(&mut app, latest.prepare(FIXTURE.to_vec()));
    app.apply_ready(handle, second).unwrap();
    let accepted = app.projection(handle).unwrap();
    complete(&mut app, old.prepare(FIXTURE.to_vec()));
    assert!(app.apply_ready(handle, first).is_err());
    assert_eq!(app.projection(handle).unwrap(), accepted);
    assert!(
        app.begin_view_source(handle, ctx(&app, handle), 2, resource(3))
            .is_err()
    );
    retire(&mut app);
    let outcome = app
        .begin_view_source(handle, ctx(&app, handle), 3, resource(3))
        .unwrap();
    let Some(HostEffect::WorkQueued(third)) = outcome.effect else {
        panic!()
    };
    let latest = read(&mut app);
    complete(&mut app, latest.prepare(vec![0]));
    app.apply_ready(handle, third).unwrap();
    assert_eq!(app.projection(handle).unwrap().editor, accepted.editor);
    assert!(app.projection(handle).unwrap().save.destination().is_none());
}

#[test]
fn retiring_old_source_preserves_newer_pending_continuation() {
    let (mut app, handle) = app(AppLimits::default(), false);
    let outcome = app
        .begin_view_source(handle, ctx(&app, handle), 1, resource(1))
        .unwrap();
    let Some(HostEffect::WorkQueued(old)) = outcome.effect else {
        panic!()
    };
    let old_work = read(&mut app);
    let outcome = app
        .begin_view_source(handle, ctx(&app, handle), 2, resource(2))
        .unwrap();
    let Some(HostEffect::WorkQueued(new)) = outcome.effect else {
        panic!()
    };
    let new_work = read(&mut app);
    let pending = app.projection(handle).unwrap().continuation;
    complete(&mut app, old_work.prepare(FIXTURE.to_vec()));
    assert!(app.apply_ready(handle, old).is_err());
    retire(&mut app);
    assert_eq!(app.projection(handle).unwrap().continuation, pending);
    complete(&mut app, new_work.prepare(FIXTURE.to_vec()));
    app.apply_ready(handle, new).unwrap();
    assert_eq!(app.projection(handle).unwrap().file, FilePhase::Idle);
    assert_eq!(app.projection(handle).unwrap().editor.document.count, 1);
}
