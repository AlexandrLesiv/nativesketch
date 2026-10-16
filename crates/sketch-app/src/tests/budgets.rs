use super::*;
// FFI-001: worker-held buffers count, rejected transfers retain caller ownership.
#[test]
fn worker_held_permit_blocks_new_buffer_and_releases_on_actual_completion() {
    let limits = AppLimits {
        buffers: 1,
        ..AppLimits::default()
    };
    let (mut app, handle) = app(limits, true);
    let first = save(&mut app, handle, resource(1), true);
    let work = app.take_work().unwrap().unwrap();
    assert_eq!(app.stats().buffers, 1);
    assert_eq!(app.stats().buffer_bytes, limits.document_bytes);
    let second = save(&mut app, handle, resource(2), true);
    assert!(app.take_work().unwrap().is_none());
    let WorkItem::Write(work) = work else {
        panic!()
    };
    let encoded = work.encode().unwrap();
    assert_eq!(app.stats().buffer_bytes, encoded.bytes().len());
    let result = written(encoded);
    assert_eq!(app.stats().buffers, 0);
    complete(&mut app, result);
    app.apply_ready(handle, first).unwrap();
    let encoded = write(&mut app);
    complete(&mut app, written(encoded));
    app.apply_ready(handle, second).unwrap();
}

#[test]
fn operation_and_resource_limits_reject_before_accepting_lease_or_changing_state() {
    let limits = AppLimits {
        operations: 1,
        destinations: 1,
        ..AppLimits::default()
    };
    let (mut app, handle) = app(limits, true);
    let first = save(&mut app, handle, resource(1), true);
    let before = app.projection(handle).unwrap();
    assert!(matches!(
        app.dispatch(handle, ctx(&app, handle), Intent::File(FileIntent::SaveAs)),
        Err(AppError::Capacity)
    ));
    assert_eq!(app.projection(handle).unwrap(), before);
    app.cancel_operation(first).unwrap();
    app.apply_ready(handle, first).unwrap();
    let outcome = run(&mut app, handle, Intent::File(FileIntent::SaveAs));
    let Some(HostEffect::ChooseDestination { token, .. }) = outcome.effect else {
        panic!()
    };
    let before = app.projection(handle).unwrap();
    assert!(matches!(
        app.supply_resource(handle, ctx(&app, handle), token, resource(2)),
        Err(AppError::Capacity)
    ));
    assert_eq!(app.projection(handle).unwrap(), before);
    assert_eq!(
        app.take_release().unwrap().descriptor.resource,
        ProviderResourceId(1)
    );
    assert!(app.take_release().is_none());
    app.supply_resource(handle, ctx(&app, handle), token, resource(2))
        .unwrap();
}

#[test]
fn oversized_encoding_and_read_fail_without_install_or_save_acknowledgment() {
    let limits = AppLimits {
        document_bytes: 24,
        ..AppLimits::default()
    };
    let (mut app, handle) = app(limits, true);
    stroke(&mut app, handle);
    let id = save(&mut app, handle, resource(1), true);
    let Some(WorkItem::Write(work)) = app.take_work().unwrap() else {
        panic!()
    };
    let result = work.encode().unwrap_err();
    complete(&mut app, result);
    let outcome = app.apply_ready(handle, id).unwrap();
    assert_eq!(
        outcome.save.unwrap().result,
        SaveResult::Failed(StorageFailure::Capacity)
    );
    assert!(outcome.projection.editor.document.dirty);
    assert_eq!(app.stats().buffers, 0);
    let (mut viewer, handle) = app_with_limits(limits);
    let outcome = viewer
        .begin_view_source(handle, ctx(&viewer, handle), 1, resource(1))
        .unwrap();
    let Some(HostEffect::WorkQueued(id)) = outcome.effect else {
        panic!()
    };
    let work = read(&mut viewer);
    assert_eq!(work.maximum_bytes(), 24);
    complete(&mut viewer, work.prepare(vec![0; 25]));
    viewer.apply_ready(handle, id).unwrap();
    assert_eq!(viewer.projection(handle).unwrap().editor.document.count, 0);
    assert_eq!(viewer.stats().buffers, 0);
}
fn app_with_limits(limits: AppLimits) -> (PortableApp, SessionHandle) {
    super::app(limits, false)
}
