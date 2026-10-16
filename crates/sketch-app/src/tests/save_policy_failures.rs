use super::{save_policy::*, *};
// SAVE-001/002: chooser completion never acknowledges subsequent edits or uncertain persistence.
#[test]
fn failed_uncertain_cancelled_and_older_chosen_save_cannot_authorize_close() {
    for mode in 0..4 {
        let (mut app, h) = bound();
        let old = app.projection(h).unwrap().save.destination();
        let token = guarded(&mut app, h, NavigationIntent::Close);
        let op = choose(&mut app, h, token);
        if mode == 2 {
            app.cancel_operation(op).unwrap();
        } else {
            let work = write(&mut app);
            let result = match mode {
                0 => work
                    .finish(WriteOutcome::Failed(StorageFailure::Write))
                    .unwrap(),
                1 => work
                    .finish(WriteOutcome::Uncertain(StorageFailure::Replace))
                    .unwrap(),
                _ => {
                    stroke(&mut app, h);
                    written(work)
                }
            };
            complete(&mut app, result);
        }
        let outcome = app.apply_ready(h, op).unwrap();
        assert!(outcome.projection.editor.document.dirty);
        assert_eq!(app.stats().outstanding_saves, 0);
        assert_eq!(app.stats().operations, 0);
        assert_eq!(
            app.take_release().unwrap().descriptor.resource,
            ProviderResourceId(if mode == 3 { 1 } else { 2 })
        );
        assert!(app.take_release().is_none());
        assert!(!matches!(
            outcome.effect,
            Some(HostEffect::NavigationReady(_) | HostEffect::CloseConfirmed)
        ));
        if mode == 3 {
            let Some(HostEffect::ConfirmUnsaved { token, .. }) = outcome.effect else {
                panic!()
            };
            let reply = app
                .reply(
                    h,
                    ctx(&app, h),
                    token,
                    EffectDecision::Unsaved(UnsavedChoice::Save),
                )
                .unwrap();
            assert!(matches!(
                reply.effect,
                Some(HostEffect::ChooseDestination { .. })
            ));
        } else {
            assert_eq!(outcome.projection.save.destination(), old);
            assert_eq!(outcome.projection.file, FilePhase::Idle);
            guarded(&mut app, h, NavigationIntent::Close);
        }
    }
}
