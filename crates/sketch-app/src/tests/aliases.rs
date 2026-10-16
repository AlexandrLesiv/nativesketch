use super::*;
// FFI-001 / SAVE-003: physical alias identity is distinct from exact access lease ownership.
#[test]
fn repeated_same_file_save_as_and_exports_release_only_obsolete_exact_leases() {
    let limits = AppLimits {
        destinations: 2,
        ..AppLimits::default()
    };
    let (mut app, handle) = app(limits, true);
    let initial = save(&mut app, handle, resource(1), false);
    let encoded = write(&mut app);
    complete(&mut app, written(encoded));
    app.apply_ready(handle, initial).unwrap();
    let mut bound = ProviderResourceId(1);
    for id in 2..=10 {
        let mut alias = resource(id);
        alias.canonical = ProviderResourceId(1);
        let export = id % 2 == 0;
        let operation = save(&mut app, handle, alias, export);
        let encoded = write(&mut app);
        complete(&mut app, written(encoded));
        assert!(app.take_release().is_none()); // result pending still owns its lease
        app.apply_ready(handle, operation).unwrap();
        let released = app.take_release().unwrap();
        assert_eq!(
            released.descriptor.resource,
            if export { alias.resource } else { bound }
        );
        if !export {
            bound = alias.resource;
        }
        assert!(app.take_release().is_none());
        assert_eq!(app.stats().resource_leases, 1);
    }
    let outcome = run(&mut app, handle, Intent::File(FileIntent::Save));
    let Some(HostEffect::WorkQueued(id)) = outcome.effect else {
        panic!()
    };
    let encoded = write(&mut app);
    assert_eq!(encoded.info().resource.unwrap().resource, bound);
    complete(&mut app, written(encoded));
    app.apply_ready(handle, id).unwrap();
    assert!(app.take_release().is_none());
}
#[test]
fn old_binding_capability_survives_failed_or_cancelled_alias_save_as() {
    let (mut app, handle) = app(AppLimits::default(), true);
    let initial = save(&mut app, handle, resource(1), false);
    let encoded = write(&mut app);
    complete(&mut app, written(encoded));
    app.apply_ready(handle, initial).unwrap();
    for id in 2..=3 {
        let mut alias = resource(id);
        alias.canonical = ProviderResourceId(1);
        let operation = save(&mut app, handle, alias, false);
        if id == 2 {
            let encoded = write(&mut app);
            complete(
                &mut app,
                encoded
                    .finish(WriteOutcome::Failed(StorageFailure::Access))
                    .unwrap(),
            );
        } else {
            app.cancel_operation(operation).unwrap();
        }
        app.apply_ready(handle, operation).unwrap();
        assert_eq!(
            app.take_release().unwrap().descriptor.resource,
            ProviderResourceId(id)
        );
        assert!(app.take_release().is_none());
    }
    let outcome = run(&mut app, handle, Intent::File(FileIntent::Save));
    let Some(HostEffect::WorkQueued(id)) = outcome.effect else {
        panic!()
    };
    let encoded = write(&mut app);
    assert_eq!(
        encoded.info().resource.unwrap().resource,
        ProviderResourceId(1)
    );
    complete(&mut app, written(encoded));
    app.apply_ready(handle, id).unwrap();
}
