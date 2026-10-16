use super::*;
pub(super) fn source(
    app: &mut ApplicationSession,
    queue: &mut SaveCoordinator,
    generation: u64,
) -> ReplacementTicket {
    let outcome = run(
        app,
        queue,
        ApplicationCommand::ViewSource {
            source: DestinationId(99),
            generation,
        },
    );
    let Some(ApplicationEffect::PrepareReplacement(ticket)) = outcome.effect else {
        panic!()
    };
    assert_eq!(
        ticket.source(),
        ReplacementSource::ViewSource(DestinationId(99))
    );
    ticket
}
pub(super) fn candidate(ticket: ReplacementTicket) -> PreparedReplacement {
    ticket
        .prepare(Some(ValidatedDocument::decode(FIXTURE).unwrap()))
        .unwrap()
}
// VIEW-001, ASYNC-001: issued latest source only, preserving generation after install.
#[test]
fn viewer_source_installs_latest_only_and_never_creates_save_binding() {
    let mut app = app(false, false);
    let mut queue = queue();
    let mut older = candidate(source(&mut app, &mut queue, 1));
    let mut newer = candidate(source(&mut app, &mut queue, 2));
    let before = app.projection();
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::Install(&mut older)
        )
        .is_err()
    );
    assert!(!older.is_consumed());
    assert_eq!(app.projection(), before);
    let outcome = run(
        &mut app,
        &mut queue,
        ApplicationCommand::Install(&mut newer),
    );
    assert!(newer.is_consumed());
    assert!(outcome.retired.is_some());
    assert_ne!(outcome.projection.editor.session, before.editor.session);
    assert_eq!(outcome.projection.save.destination(), None);
    assert!(!outcome.projection.editor.document.dirty);
    for generation in [0, 1, 2] {
        assert!(
            app.dispatch(
                &mut queue,
                context(&app),
                ApplicationCommand::ViewSource {
                    source: DestinationId(99),
                    generation,
                }
            )
            .is_err()
        );
    }
    let mut third = candidate(source(&mut app, &mut queue, 3));
    run(
        &mut app,
        &mut queue,
        ApplicationCommand::Install(&mut third),
    );
    assert_eq!(bytes(&app), FIXTURE);
}
// LOAD-001, SOURCE-001: failed newest retrieval retains last displayed valid document.
#[test]
fn corrupt_latest_and_cancelled_source_cannot_clear_or_replace_displayed_document() {
    let mut app = app(false, false);
    let mut queue = queue();
    let original = bytes(&app);
    let old = source(&mut app, &mut queue, 1);
    let latest = source(&mut app, &mut queue, 2);
    assert!(ValidatedDocument::decode(b"corrupt").is_err());
    let mut failure = Some(latest.fail(StorageFailure::Encode));
    let result = run(
        &mut app,
        &mut queue,
        ApplicationCommand::ReplacementFailed(&mut failure),
    );
    assert!(matches!(result.effect, Some(ApplicationEffect::Failure(_))));
    assert!(failure.is_none());
    assert_eq!(bytes(&app), original);
    let mut old = candidate(old);
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::Install(&mut old)
        )
        .is_err()
    );
    let cancelled = source(&mut app, &mut queue, 3);
    let token = cancelled.token();
    reply(&mut app, &mut queue, token, EffectDecision::Cancel);
    let mut cancelled = candidate(cancelled);
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::Install(&mut cancelled)
        )
        .is_err()
    );
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::ViewSource {
                source: DestinationId(99),
                generation: 3,
            }
        )
        .is_err()
    );
    assert_eq!(bytes(&app), original);
}
#[test]
fn stale_source_failure_cannot_consume_latest_ticket_and_max_generation_never_wraps() {
    let mut app = app(false, false);
    let mut queue = queue();
    let old = source(&mut app, &mut queue, 1);
    let mut latest = candidate(source(&mut app, &mut queue, u64::MAX));
    let mut failure = Some(old.fail(StorageFailure::Access));
    let before = app.projection();
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::ReplacementFailed(&mut failure)
        )
        .is_err()
    );
    assert!(failure.is_some());
    assert_eq!(app.projection(), before);
    run(
        &mut app,
        &mut queue,
        ApplicationCommand::Install(&mut latest),
    );
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::ViewSource {
                source: DestinationId(99),
                generation: 0,
            }
        )
        .is_err()
    );
}
