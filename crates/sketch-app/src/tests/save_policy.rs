use super::*;
pub(super) fn bound() -> (PortableApp, SessionHandle) {
    let mut app = PortableApp::new(AppLimits::default()).unwrap();
    let h = app
        .create_session_with_save_policy(config(true), SaveDestinationPolicy::AlwaysChoose)
        .unwrap();
    stroke(&mut app, h);
    let op = save(&mut app, h, resource(1), false);
    let work = write(&mut app);
    complete(&mut app, written(work));
    app.apply_ready(h, op).unwrap();
    stroke(&mut app, h);
    (app, h)
}
pub(super) fn guarded(
    app: &mut PortableApp,
    h: SessionHandle,
    nav: NavigationIntent,
) -> EffectToken {
    let Some(HostEffect::ConfirmUnsaved { token, .. }) =
        run(app, h, Intent::File(FileIntent::Navigate(nav))).effect
    else {
        panic!()
    };
    let outcome = app
        .reply(
            h,
            ctx(app, h),
            token,
            EffectDecision::Unsaved(UnsavedChoice::Save),
        )
        .unwrap();
    let Some(HostEffect::ChooseDestination {
        token,
        purpose: SavePurpose::SaveAs,
    }) = outcome.effect
    else {
        panic!("prior binding must not bypass chooser")
    };
    token
}
pub(super) fn choose(app: &mut PortableApp, h: SessionHandle, token: EffectToken) -> OperationId {
    let outcome = app
        .supply_resource(h, ctx(app, h), token, resource(2))
        .unwrap();
    let Some(HostEffect::WorkQueued(op)) = outcome.effect else {
        panic!()
    };
    op
}
// SAVE-001/003: reproduce Linux/Android Save after a prior successful destination binding.
#[test]
fn bound_direct_and_guarded_saves_choose_again_and_cancel_without_issuing_writes() {
    for nav in [
        NavigationIntent::New,
        NavigationIntent::Open,
        NavigationIntent::Close,
    ] {
        let (mut app, h) = bound();
        let before = app.projection(h).unwrap();
        let Some(HostEffect::ChooseDestination {
            token,
            purpose: SavePurpose::SaveAs,
        }) = run(&mut app, h, Intent::File(FileIntent::Save)).effect
        else {
            panic!()
        };
        assert!(app.take_work().unwrap().is_none());
        app.reply(h, ctx(&app, h), token, EffectDecision::Cancel)
            .unwrap();
        assert_eq!(app.projection(h).unwrap(), before);
        let token = guarded(&mut app, h, nav);
        assert!(app.take_work().unwrap().is_none());
        app.reply(h, ctx(&app, h), token, EffectDecision::Cancel)
            .unwrap();
        assert_eq!(app.projection(h).unwrap(), before);
        assert_eq!(app.stats().operations, 0);
    }
}
#[test]
fn chosen_destination_only_advances_navigation_after_matching_confirmed_write() {
    for nav in [
        NavigationIntent::New,
        NavigationIntent::Open,
        NavigationIntent::Close,
    ] {
        let (mut app, h) = bound();
        let old = app.projection(h).unwrap().save.destination();
        let token = guarded(&mut app, h, nav);
        let op = choose(&mut app, h, token);
        assert_eq!(app.projection(h).unwrap().save.destination(), old);
        let work = write(&mut app);
        assert_eq!(
            work.info().resource.unwrap().resource,
            ProviderResourceId(2)
        );
        complete(&mut app, written(work));
        let outcome = app.apply_ready(h, op).unwrap();
        assert!(!outcome.projection.editor.document.dirty);
        assert_ne!(outcome.projection.save.destination(), old);
        match nav {
            NavigationIntent::New => {
                assert!(matches!(outcome.effect, Some(HostEffect::WorkQueued(_))))
            }
            NavigationIntent::Open => {
                assert!(matches!(outcome.effect, Some(HostEffect::ChooseSource(_))))
            }
            NavigationIntent::Close => assert!(matches!(
                outcome.effect,
                Some(HostEffect::NavigationReady(_))
            )),
            _ => unreachable!(),
        }
    }
}
