use super::*;
#[test]
fn stale_context_rejection_retains_trusted_completion_for_correct_owner() {
    let mut app = app(true, true);
    let mut queue = queue();
    save_as(&mut app, &mut queue, DestinationId(1));
    let mut completion = Some(complete(&mut queue, written()));
    let other = super::app(true, true);
    let before = app.projection();
    assert!(
        app.dispatch(
            &mut queue,
            context(&other),
            ApplicationCommand::CompleteSave(&mut completion)
        )
        .is_err()
    );
    assert!(completion.is_some());
    assert_eq!(app.projection(), before);
    let result = run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut completion),
    );
    assert!(completion.is_none());
    assert_eq!(
        result.save.unwrap().acknowledgement,
        Acknowledgement::SavedCurrent
    );
    assert_eq!(app.projection().outstanding_saves, 0);
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::CompleteSave(&mut completion)
        )
        .is_err()
    );
}
#[test]
fn failed_navigation_save_preserves_edits_and_cancels_navigation_with_failure_details() {
    let mut app = app(true, true);
    let mut queue = queue();
    stroke(&mut app, &mut queue);
    let original = bytes(&app);
    let ApplicationEffect::ConfirmUnsaved { token, .. } = file(
        &mut app,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::Close),
    ) else {
        panic!()
    };
    let ApplicationEffect::ChooseDestination { token, .. } = reply(
        &mut app,
        &mut queue,
        token,
        EffectDecision::Unsaved(UnsavedChoice::Save),
    ) else {
        panic!()
    };
    reply(
        &mut app,
        &mut queue,
        token,
        EffectDecision::Destination(DestinationId(1)),
    );
    assert!(app.projection().continuation.is_some());
    let mut completion = Some(complete(
        &mut queue,
        WriteOutcome::Failed(StorageFailure::Capacity),
    ));
    let outcome = run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut completion),
    );
    assert!(matches!(
        outcome.effect,
        Some(ApplicationEffect::Failure(ApplicationFailure::Write(
            StorageFailure::Capacity
        )))
    ));
    assert_eq!(
        outcome.save.unwrap().result,
        SaveResult::Failed(StorageFailure::Capacity)
    );
    assert!(app.projection().navigation.is_none());
    assert!(app.projection().editor.document.dirty);
    assert_eq!(bytes(&app), original);
}
#[test]
fn source_choice_cannot_authorize_edits_committed_while_panel_was_pending() {
    let mut app = app(true, true);
    let mut queue = queue();
    let ApplicationEffect::ChooseSource(token) = file(
        &mut app,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::Open),
    ) else {
        panic!()
    };
    stroke(&mut app, &mut queue);
    let before = app.projection();
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::ResolveEffect(EffectReply {
                token,
                decision: EffectDecision::Source(DestinationId(7))
            })
        )
        .is_err()
    );
    assert_eq!(app.projection(), before);
    assert!(matches!(
        reply(&mut app, &mut queue, token, EffectDecision::Cancel),
        ApplicationEffect::Cancelled
    ));
}
#[test]
fn dropped_completion_keeps_visible_outstanding_obligation() {
    let mut app = app(true, true);
    let mut queue = queue();
    save_as(&mut app, &mut queue, DestinationId(1));
    let _undelivered = complete(&mut queue, written());
    assert_eq!(queue.outstanding(), 0);
    assert_eq!(app.projection().outstanding_saves, 1);
    assert!(matches!(
        file(
            &mut app,
            &mut queue,
            FileIntent::Navigate(NavigationIntent::Close)
        ),
        ApplicationEffect::AwaitWrites(_)
    ));
}
#[test]
fn effect_from_another_session_cannot_cancel_pending_navigation_or_ink() {
    let mut first = app(true, true);
    let mut second = app(true, true);
    let mut queue = queue();
    let ApplicationEffect::ChooseSource(token) = file(
        &mut first,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::Open),
    ) else {
        panic!()
    };
    file(
        &mut second,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::Open),
    );
    draft(&mut second, &mut queue);
    let before = second.projection();
    assert!(
        second
            .dispatch(
                &mut queue,
                context(&second),
                ApplicationCommand::ResolveEffect(EffectReply {
                    token,
                    decision: EffectDecision::Cancel
                })
            )
            .is_err()
    );
    assert_eq!(second.projection(), before);
}
