use super::*;
// SAVE-001/003: immutable snapshots, global write order, no old-revision clean claim.
#[test]
fn older_snapshot_cannot_mark_new_edits_clean_and_aliases_serialize() {
    let (mut app, handle) = app(AppLimits::default(), true);
    let bound = save(&mut app, handle, resource(1), false);
    let encoded = write(&mut app);
    complete(&mut app, written(encoded));
    app.apply_ready(handle, bound).unwrap();
    stroke(&mut app, handle);
    let outcome = run(&mut app, handle, Intent::File(FileIntent::Save));
    let Some(HostEffect::WorkQueued(first)) = outcome.effect else {
        panic!()
    };
    let encoded = write(&mut app);
    let bytes = encoded.bytes().to_vec();
    stroke(&mut app, handle);
    let mut alias = resource(2);
    alias.canonical = ProviderResourceId(1);
    let second = save(&mut app, handle, alias, true);
    assert!(app.take_work().unwrap().is_none());
    assert_eq!(app.stats().outstanding_saves, 2);
    complete(&mut app, written(encoded));
    let outcome = app.apply_ready(handle, first).unwrap();
    assert_eq!(
        outcome.save.unwrap().acknowledgement,
        Acknowledgement::SavedOlder
    );
    assert!(outcome.projection.editor.document.dirty);
    let encoded = write(&mut app);
    assert_ne!(encoded.bytes(), bytes);
    complete(&mut app, written(encoded));
    let outcome = app.apply_ready(handle, second).unwrap();
    assert!(outcome.projection.editor.document.dirty);
    assert_eq!(app.stats().outstanding_saves, 0);
}

#[test]
fn save_as_failure_retains_binding_and_uncertain_partial_write_is_not_clean() {
    let (mut app, handle) = app(AppLimits::default(), true);
    let first = save(&mut app, handle, resource(1), false);
    let encoded = write(&mut app);
    complete(&mut app, written(encoded));
    let bound = app
        .apply_ready(handle, first)
        .unwrap()
        .projection
        .save
        .destination();
    let second = save(&mut app, handle, resource(2), false);
    let encoded = write(&mut app);
    complete(
        &mut app,
        encoded
            .finish(WriteOutcome::Failed(StorageFailure::Access))
            .unwrap(),
    );
    let outcome = app.apply_ready(handle, second).unwrap();
    assert_eq!(outcome.projection.save.destination(), bound);
    assert!(!outcome.projection.editor.document.dirty);
    let outcome = run(&mut app, handle, Intent::File(FileIntent::Save));
    let Some(HostEffect::WorkQueued(third)) = outcome.effect else {
        panic!()
    };
    let encoded = write(&mut app);
    complete(
        &mut app,
        encoded
            .finish(WriteOutcome::Uncertain(StorageFailure::Write))
            .unwrap(),
    );
    let outcome = app.apply_ready(handle, third).unwrap();
    assert_eq!(
        outcome.save.unwrap().acknowledgement,
        Acknowledgement::Failed
    );
    assert!(outcome.projection.editor.document.dirty);
    assert_eq!(
        app.take_release().unwrap().descriptor.resource,
        ProviderResourceId(2)
    );
    assert!(app.take_release().is_none());
}

#[test]
fn active_cancel_holds_order_until_real_receipt_and_unconfirmed_rejection_keeps_bytes() {
    let (mut app, handle) = app(AppLimits::default(), true);
    stroke(&mut app, handle);
    let first = save(&mut app, handle, resource(1), false);
    let encoded = write(&mut app);
    let bytes = encoded.bytes().to_vec();
    let (error, encoded) = encoded
        .finish(WriteOutcome::DispatchedUnconfirmed)
        .unwrap_err();
    assert_eq!(error, AppError::CapabilityDenied);
    assert_eq!(encoded.bytes(), bytes);
    app.cancel_operation(first).unwrap();
    let second = save(&mut app, handle, resource(2), true);
    assert!(app.take_work().unwrap().is_none());
    complete(&mut app, written(encoded));
    let outcome = app.apply_ready(handle, first).unwrap();
    assert_eq!(
        outcome.save.unwrap().acknowledgement,
        Acknowledgement::Cancelled
    );
    assert!(outcome.projection.editor.document.dirty);
    assert!(outcome.projection.save.destination().is_none());
    app.cancel_operation(second).unwrap();
    let outcome = app.apply_ready(handle, second).unwrap();
    assert_eq!(
        outcome.save.unwrap().result,
        SaveResult::CancelledBeforeStart
    );
    assert!(app.take_work().unwrap().is_none());
}

#[test]
fn unconfirmed_export_settles_barrier_but_still_prompts_for_dirty_navigation() {
    let (mut app, handle) = app(AppLimits::default(), true);
    stroke(&mut app, handle);
    let mut download = resource(1);
    download.access.confirmed_write = false;
    let operation = save(&mut app, handle, download, true);
    let encoded = write(&mut app);
    let (error, encoded) = encoded
        .finish(WriteOutcome::Written(Durability::ProviderConfirmed))
        .unwrap_err();
    assert_eq!(error, AppError::CapabilityDenied);
    let outcome = run(
        &mut app,
        handle,
        Intent::File(FileIntent::Navigate(NavigationIntent::New)),
    );
    assert!(matches!(outcome.effect, Some(HostEffect::AwaitWrites(_))));
    complete(
        &mut app,
        encoded.finish(WriteOutcome::DispatchedUnconfirmed).unwrap(),
    );
    let outcome = app.apply_ready(handle, operation).unwrap();
    assert_eq!(
        outcome.save.unwrap().acknowledgement,
        Acknowledgement::CopyDispatched
    );
    assert!(matches!(
        outcome.effect,
        Some(HostEffect::ConfirmUnsaved { .. })
    ));
    assert!(outcome.projection.editor.document.dirty);
    assert!(outcome.projection.save.destination().is_none());
}
