use super::*;
#[test]
fn stale_install_preserves_candidate_document_and_pending_state() {
    let mut app = app(true, true);
    let mut queue = queue();
    let mut candidate = prepared(&mut app, &mut queue, DestinationId(7));
    stroke(&mut app, &mut queue);
    let before = app.projection();
    let data = bytes(&app);
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::Install(&mut candidate)
        )
        .is_err()
    );
    assert!(!candidate.is_consumed());
    assert_eq!(app.projection(), before);
    assert_eq!(bytes(&app), data);
}
#[test]
fn atomic_open_binds_clears_history_selection_and_preserves_current_settings() {
    let mut app = app(true, true);
    let mut queue = queue();
    let original = app.projection().editor.session;
    stroke(&mut app, &mut queue);
    save_as(&mut app, &mut queue, DestinationId(1));
    let mut completion = Some(complete(&mut queue, written()));
    run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut completion),
    );
    assert!(app.projection().editor.document.can_undo);
    let mut candidate = prepared(&mut app, &mut queue, DestinationId(7));
    edit(&mut app, &mut queue, EditorCommand::SetBrush(brush(19.0)));
    edit(&mut app, &mut queue, EditorCommand::SetTool(ToolId::Select));
    edit(&mut app, &mut queue, EditorCommand::SelectCycle(1));
    assert_ne!(app.projection().editor.document.selected, 0);
    let outcome = run(
        &mut app,
        &mut queue,
        ApplicationCommand::Install(&mut candidate),
    );
    assert_eq!(outcome.retired.unwrap().view_state().session, original);
    assert!(candidate.is_consumed());
    let projection = app.projection();
    assert_ne!(projection.editor.session, original);
    assert_eq!(projection.save.destination(), Some(DestinationId(7)));
    assert_eq!(projection.editor.tool, ToolId::Select);
    assert_eq!(projection.editor.brush, brush(19.0));
    assert!(!projection.editor.document.dirty);
    assert!(!projection.editor.document.can_undo);
    assert_eq!(projection.editor.document.selected, 0);
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::Install(&mut candidate)
        )
        .is_err()
    );
}
#[test]
fn recovered_copy_is_dirty_unbound_and_cancelled_read_cannot_replace() {
    let mut app = app(true, true);
    let mut queue = queue();
    let ApplicationEffect::PrepareReplacement(ticket) = file(
        &mut app,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::Recover(RecoverySourceId(9))),
    ) else {
        panic!()
    };
    let token = ticket.token();
    let mut candidate = ticket
        .prepare(Some(ValidatedDocument::decode(FIXTURE).unwrap()))
        .unwrap();
    reply(&mut app, &mut queue, token, EffectDecision::Cancel);
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::Install(&mut candidate)
        )
        .is_err()
    );
    assert!(!candidate.is_consumed());
    let ApplicationEffect::PrepareReplacement(ticket) = file(
        &mut app,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::Recover(RecoverySourceId(9))),
    ) else {
        panic!()
    };
    let mut candidate = ticket
        .prepare(Some(ValidatedDocument::decode(FIXTURE).unwrap()))
        .unwrap();
    run(
        &mut app,
        &mut queue,
        ApplicationCommand::Install(&mut candidate),
    );
    assert!(app.projection().editor.document.dirty);
    assert_eq!(app.projection().save.destination(), None);
}
#[test]
fn failed_preparation_preserves_live_document_and_reports_issued_failure() {
    let mut app = app(true, true);
    let mut queue = queue();
    let ApplicationEffect::PrepareReplacement(ticket) = file(
        &mut app,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::New),
    ) else {
        panic!()
    };
    let failure = ticket
        .prepare(Some(ValidatedDocument::decode(FIXTURE).unwrap()))
        .unwrap_err();
    let old = bytes(&app);
    let outcome = run(
        &mut app,
        &mut queue,
        ApplicationCommand::ReplacementFailed(failure),
    );
    assert!(matches!(
        outcome.effect,
        Some(ApplicationEffect::Failure(_))
    ));
    assert_eq!(bytes(&app), old);
    assert_eq!(app.projection().file, FilePhase::Idle);
}
