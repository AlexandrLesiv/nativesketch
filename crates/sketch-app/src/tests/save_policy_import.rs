use super::*;
// SAVE-001 / ASYNC-001: read-only imported grants must not block a fresh destination chooser.
#[test]
fn imported_read_only_binding_uses_policy_for_direct_and_guarded_save_preflight() {
    for source_write in [false, true] {
        for policy in [
            SaveDestinationPolicy::ReuseBinding,
            SaveDestinationPolicy::AlwaysChoose,
        ] {
            for guarded in [false, true] {
                let mut app = PortableApp::new(AppLimits::default()).unwrap();
                let h = app
                    .create_session_with_save_policy(config(true), policy)
                    .unwrap();
                let mut source = resource(1);
                source.access.write = source_write;
                source.access.confirmed_write = false;
                let op = open(&mut app, h, source);
                let work = read(&mut app);
                complete(&mut app, work.prepare(FIXTURE.to_vec()));
                app.apply_ready(h, op).unwrap();
                retire(&mut app);
                stroke(&mut app, h);
                let token = if guarded {
                    let Some(HostEffect::ConfirmUnsaved { token, .. }) = run(
                        &mut app,
                        h,
                        Intent::File(FileIntent::Navigate(NavigationIntent::Close)),
                    )
                    .effect
                    else {
                        panic!()
                    };
                    Some(token)
                } else {
                    None
                };
                let before = app.projection(h).unwrap();
                let outcome = if let Some(token) = token {
                    app.reply(
                        h,
                        ctx(&app, h),
                        token,
                        EffectDecision::Unsaved(UnsavedChoice::Save),
                    )
                } else {
                    app.dispatch(h, ctx(&app, h), Intent::File(FileIntent::Save))
                };
                if policy == SaveDestinationPolicy::ReuseBinding {
                    assert!(matches!(outcome, Err(AppError::CapabilityDenied)));
                    assert_eq!(app.projection(h).unwrap(), before);
                } else {
                    let outcome = outcome.unwrap();
                    let Some(HostEffect::ChooseDestination {
                        token,
                        purpose: SavePurpose::SaveAs,
                    }) = outcome.effect
                    else {
                        panic!()
                    };
                    assert_eq!(
                        outcome.projection.save.destination(),
                        before.save.destination()
                    );
                    assert!(app.take_work().unwrap().is_none());
                    let op = super::save_policy::choose(&mut app, h, token);
                    let work = write(&mut app);
                    assert_eq!(
                        work.info().resource.unwrap().resource,
                        ProviderResourceId(2)
                    );
                    complete(&mut app, written(work));
                    let outcome = app.apply_ready(h, op).unwrap();
                    assert!(!outcome.projection.editor.document.dirty);
                    assert_ne!(
                        outcome.projection.save.destination(),
                        before.save.destination()
                    );
                    assert_eq!(
                        app.take_release().unwrap().descriptor.resource,
                        ProviderResourceId(1)
                    );
                    assert!(app.take_release().is_none());
                }
                assert_eq!(app.stats().operations, 0);
                assert_eq!(app.stats().outstanding_saves, 0);
            }
        }
    }
}
