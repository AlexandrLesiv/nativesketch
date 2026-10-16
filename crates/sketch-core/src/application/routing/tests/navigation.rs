use super::*;
#[test]
fn undelivered_save_completion_cannot_be_bypassed_by_navigation_continue() {
    let mut app = app(true, true);
    let mut queue = queue();
    save_as(&mut app, &mut queue, DestinationId(1));
    let completion = complete(&mut queue, written());
    assert_eq!(queue.outstanding(), 0);
    let ApplicationEffect::AwaitWrites(token) = file(
        &mut app,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::Close),
    ) else {
        panic!()
    };
    assert!(matches!(
        reply(&mut app, &mut queue, token, EffectDecision::Continue),
        ApplicationEffect::AwaitWrites(_)
    ));
    let result = run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut Some(completion)),
    );
    let Some(ApplicationEffect::NavigationReady(token)) = result.effect else {
        panic!()
    };
    assert!(matches!(
        reply(&mut app, &mut queue, token, EffectDecision::ConfirmClose),
        ApplicationEffect::CloseConfirmed
    ));
}
#[test]
fn dirty_edits_after_save_and_after_close_authorization_require_new_decision() {
    let mut app = app(true, true);
    let mut queue = queue();
    stroke(&mut app, &mut queue);
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
    stroke(&mut app, &mut queue);
    let completion = complete(&mut queue, written());
    let result = run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut Some(completion)),
    );
    let Some(ApplicationEffect::ConfirmUnsaved { token, .. }) = result.effect else {
        panic!()
    };
    let ApplicationEffect::NavigationReady(token) = reply(
        &mut app,
        &mut queue,
        token,
        EffectDecision::Unsaved(UnsavedChoice::Discard),
    ) else {
        panic!()
    };
    stroke(&mut app, &mut queue);
    assert!(matches!(
        reply(&mut app, &mut queue, token, EffectDecision::ConfirmClose),
        ApplicationEffect::ConfirmUnsaved { .. }
    ));
}
#[test]
fn old_discard_decision_does_not_authorize_newer_edits_or_clear_history() {
    let mut app = app(true, true);
    let mut queue = queue();
    stroke(&mut app, &mut queue);
    let ApplicationEffect::ConfirmUnsaved { token, .. } = file(
        &mut app,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::New),
    ) else {
        panic!()
    };
    stroke(&mut app, &mut queue);
    assert!(matches!(
        reply(
            &mut app,
            &mut queue,
            token,
            EffectDecision::Unsaved(UnsavedChoice::Discard)
        ),
        ApplicationEffect::ConfirmUnsaved { .. }
    ));
    assert!(app.projection().editor.document.can_undo);
}
#[test]
fn cancelled_navigation_does_not_cancel_write_or_reopen_it_on_late_completion() {
    let mut app = app(true, true);
    let mut queue = queue();
    save_as(&mut app, &mut queue, DestinationId(1));
    let ApplicationEffect::AwaitWrites(token) = file(
        &mut app,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::Close),
    ) else {
        panic!()
    };
    reply(&mut app, &mut queue, token, EffectDecision::Cancel);
    let completion = complete(&mut queue, written());
    let outcome = run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut Some(completion)),
    );
    assert!(outcome.effect.is_none());
    assert_eq!(app.projection().save.destination(), Some(DestinationId(1)));
    assert_eq!(app.projection().file, FilePhase::Idle);
}
