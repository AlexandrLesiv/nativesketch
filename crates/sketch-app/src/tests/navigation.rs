use super::*;
// SAVE-001 / LOAD-001: failed Save decision cannot complete navigation or strand retry.
#[test]
fn failed_or_uncertain_save_before_navigation_preserves_edits_and_retry_can_discard() {
    for bound in [false, true] {
        for failure in [
            WriteOutcome::Failed(StorageFailure::Write),
            WriteOutcome::Uncertain(StorageFailure::Replace),
        ] {
            let (mut app, handle) = app(AppLimits::default(), true);
            if bound {
                let initial = save(&mut app, handle, resource(1), false);
                let encoded = write(&mut app);
                complete(&mut app, written(encoded));
                app.apply_ready(handle, initial).unwrap();
            }
            stroke(&mut app, handle);
            let before = app.projection(handle).unwrap().editor;
            let outcome = run(
                &mut app,
                handle,
                Intent::File(FileIntent::Navigate(NavigationIntent::New)),
            );
            let Some(HostEffect::ConfirmUnsaved { token, .. }) = outcome.effect else {
                panic!()
            };
            let outcome = app
                .reply(
                    handle,
                    ctx(&app, handle),
                    token,
                    EffectDecision::Unsaved(UnsavedChoice::Save),
                )
                .unwrap();
            let operation = match outcome.effect {
                Some(HostEffect::WorkQueued(id)) => id,
                Some(HostEffect::ChooseDestination { token, .. }) => {
                    let outcome = app
                        .supply_resource(handle, ctx(&app, handle), token, resource(2))
                        .unwrap();
                    let Some(HostEffect::WorkQueued(id)) = outcome.effect else {
                        panic!()
                    };
                    id
                }
                _ => panic!("issued save or destination required"),
            };
            let encoded = write(&mut app);
            complete(&mut app, encoded.finish(failure).unwrap());
            let outcome = app.apply_ready(handle, operation).unwrap();
            assert!(matches!(
                outcome.effect,
                Some(HostEffect::Failure(ApplicationFailure::Write(_)))
            ));
            assert_eq!(outcome.projection.file, FilePhase::Idle);
            assert_eq!(outcome.projection.editor.session, before.session);
            assert_eq!(
                outcome.projection.editor.document.count,
                before.document.count
            );
            assert!(outcome.projection.editor.document.dirty);
            let outcome = run(
                &mut app,
                handle,
                Intent::File(FileIntent::Navigate(NavigationIntent::New)),
            );
            let Some(HostEffect::ConfirmUnsaved { token, .. }) = outcome.effect else {
                panic!()
            };
            let outcome = app
                .reply(
                    handle,
                    ctx(&app, handle),
                    token,
                    EffectDecision::Unsaved(UnsavedChoice::Discard),
                )
                .unwrap();
            let Some(HostEffect::WorkQueued(id)) = outcome.effect else {
                panic!()
            };
            let Some(WorkItem::PrepareNew(work)) = app.take_work().unwrap() else {
                panic!()
            };
            complete(&mut app, work.prepare());
            app.apply_ready(handle, id).unwrap();
            assert_eq!(app.projection(handle).unwrap().editor.document.count, 0);
            retire(&mut app);
            assert_eq!(app.stats().outstanding_saves, 0);
        }
    }
}
