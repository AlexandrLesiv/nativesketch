use super::*;
// FFI-001 / ASYNC-001: completion provenance and teardown retain actual work ownership.
#[test]
fn wrong_owner_rejects_result_without_consuming_it() {
    let (mut app, handle) = app(AppLimits::default(), true);
    let id = save(&mut app, handle, resource(1), true);
    let encoded = write(&mut app);
    let mut result = Some(written(encoded));
    let (mut other, other_handle) = super::app(AppLimits::default(), true);
    assert!(matches!(
        other.complete_work(&mut result),
        Err(AppError::Stale)
    ));
    assert!(result.is_some());
    assert!(app.projection(other_handle).is_err());
    app.complete_work(&mut result).unwrap();
    assert!(result.is_none());
    assert!(matches!(
        app.complete_work(&mut result),
        Err(AppError::Invalid)
    ));
    app.apply_ready(handle, id).unwrap();
    assert!(app.apply_ready(handle, id).is_err());
}

#[test]
fn pristine_detach_waits_for_real_save_receipt_then_retires_and_releases_once() {
    let (mut app, handle) = app(AppLimits::default(), true);
    let id = save(&mut app, handle, resource(1), false);
    let encoded = write(&mut app);
    app.detach_session(handle).unwrap();
    assert!(!app.stats().shutdown_ready());
    assert!(app.take_release().is_none());
    assert!(
        app.dispatch(
            handle,
            ctx(&app, handle),
            Intent::Action(EditorAction::ChooseTool(ToolId::Select))
        )
        .is_err()
    );
    complete(&mut app, written(encoded));
    app.apply_ready(handle, id).unwrap();
    assert!(!app.stats().shutdown_ready());
    retire(&mut app);
    assert_eq!(
        app.take_release().unwrap().descriptor.resource,
        ProviderResourceId(1)
    );
    assert!(app.take_release().is_none());
    assert!(app.stats().shutdown_ready());
}

#[test]
fn cancelled_active_read_cannot_install_and_remains_owned_until_late_completion() {
    let (mut app, handle) = app(AppLimits::default(), false);
    let outcome = app
        .begin_view_source(handle, ctx(&app, handle), 1, resource(1))
        .unwrap();
    let Some(HostEffect::WorkQueued(id)) = outcome.effect else {
        panic!()
    };
    let work = read(&mut app);
    app.cancel_operation(id).unwrap();
    app.detach_session(handle).unwrap();
    assert_eq!(app.stats().buffers, 1);
    assert!(app.take_release().is_none());
    complete(&mut app, work.prepare(FIXTURE.to_vec()));
    assert!(app.apply_ready(handle, id).is_err());
    retire(&mut app);
    app.take_release().unwrap();
    assert!(app.stats().shutdown_ready());
}

#[test]
fn abandoned_work_does_not_claim_provider_termination_or_unlock_write_order() {
    let (mut app, handle) = app(AppLimits::default(), true);
    save(&mut app, handle, resource(1), true);
    let encoded = write(&mut app);
    save(&mut app, handle, resource(2), true);
    drop(encoded);
    assert_eq!(app.stats().buffers, 0);
    assert_eq!(app.stats().outstanding_saves, 2);
    assert!(!app.stats().shutdown_ready());
    assert!(app.take_work().unwrap().is_none());
}
