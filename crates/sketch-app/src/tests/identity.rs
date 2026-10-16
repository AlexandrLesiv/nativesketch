use super::*;
// FFI-001 / ASYNC-001: opaque IDs keep integer precision; rejected ownership is recoverable.
#[test]
fn high_bit_ids_are_exact_and_exhaustion_precedes_document_or_dialog_mutation() {
    let (mut app, _) = app(AppLimits::default(), true);
    app.next_session = (1_u64 << 63) + 1;
    let handle = app.create_session(config(true)).unwrap();
    assert_eq!(handle.get(), (1_u64 << 63) + 1);
    app.next_operation = (1_u64 << 53) + 1;
    let id = save(&mut app, handle, resource(1), true);
    assert_eq!(id.get(), (1_u64 << 53) + 2); // chooser reservation precedes work admission
    app.cancel_operation(id).unwrap();
    app.apply_ready(handle, id).unwrap();
    let before = app.projection(handle).unwrap();
    app.next_operation = u64::MAX;
    assert!(matches!(
        app.dispatch(handle, ctx(&app, handle), Intent::File(FileIntent::SaveAs)),
        Err(AppError::Capacity)
    ));
    assert_eq!(app.projection(handle).unwrap(), before);
    app.next_session = u64::MAX;
    let count = app.stats().sessions;
    assert_eq!(
        app.create_session(config(true)).unwrap_err(),
        AppError::Capacity
    );
    assert_eq!(app.stats().sessions, count);
}

#[test]
fn rejected_wrong_kind_result_keeps_real_issued_write_for_corrected_submission() {
    let (mut app, handle) = app(AppLimits::default(), true);
    let id = save(&mut app, handle, resource(1), true);
    let encoded = write(&mut app);
    let mut result = written(encoded);
    let actual_payload = std::mem::replace(&mut result.payload, crate::work::Payload::Retired);
    let mut held = Some(result);
    assert_eq!(app.complete_work(&mut held), Err(AppError::Invalid));
    assert!(held.is_some());
    assert_eq!(app.stats().outstanding_saves, 1);
    held.as_mut().unwrap().payload = actual_payload;
    app.complete_work(&mut held).unwrap();
    app.apply_ready(handle, id).unwrap();
    assert_eq!(app.stats().outstanding_saves, 0);
}

#[test]
fn stale_resource_reply_retains_host_lease_and_cancel_exhaustion_is_atomic() {
    let (mut app, handle) = app(AppLimits::default(), true);
    let outcome = run(
        &mut app,
        handle,
        Intent::File(FileIntent::Navigate(NavigationIntent::Open)),
    );
    let Some(HostEffect::ChooseSource(token)) = outcome.effect else {
        panic!()
    };
    let stale = ctx(&app, handle);
    draft(&mut app, handle);
    let before = app.projection(handle).unwrap();
    assert!(
        app.supply_resource(handle, stale, token, resource(1))
            .is_err()
    );
    assert_eq!(app.projection(handle).unwrap(), before);
    assert_eq!(app.stats().resource_leases, 0);
    assert!(app.take_release().is_none());
    run(
        &mut app,
        handle,
        Intent::Edit(EditorCommand::CancelInteraction),
    );
    let outcome = app
        .supply_resource(handle, ctx(&app, handle), token, resource(1))
        .unwrap();
    let Some(HostEffect::WorkQueued(id)) = outcome.effect else {
        panic!()
    };
    let before = app.projection(handle).unwrap();
    app.next_operation = u64::MAX;
    assert_eq!(app.cancel_operation(id), Err(AppError::Capacity));
    assert_eq!(app.projection(handle).unwrap(), before);
    assert_eq!(app.stats().operations, 1);
    assert!(app.take_release().is_none());
}

#[test]
fn issued_read_and_result_move_to_worker_without_live_app_borrow() {
    fn assert_send<T: Send>() {}
    assert_send::<WorkItem>();
    assert_send::<WorkResult>();
    assert_send::<EncodedWrite>();
    let (mut app, handle) = app(AppLimits::default(), true);
    let id = open(&mut app, handle, resource(1));
    let work = read(&mut app);
    let result = std::thread::spawn(move || work.prepare(FIXTURE.to_vec()))
        .join()
        .unwrap();
    complete(&mut app, result);
    assert_eq!(app.projection(handle).unwrap().editor.document.count, 0);
    app.apply_ready(handle, id).unwrap();
    assert_eq!(app.projection(handle).unwrap().editor.document.count, 1);
}
