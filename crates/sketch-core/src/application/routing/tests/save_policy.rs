use super::*;
fn configured(policy: SaveDestinationPolicy) -> (ApplicationSession, SaveCoordinator) {
    let mut app = ApplicationSession::new_with_save_policy(
        Capabilities {
            edit_document: true,
            export_document: true,
        },
        ToolId::Pen,
        brush(4.0),
        Some(ValidatedDocument::decode(FIXTURE).unwrap()),
        policy,
    )
    .unwrap();
    let mut queue = queue();
    save_as(&mut app, &mut queue, DestinationId(7));
    let mut completion = Some(complete(&mut queue, written()));
    run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut completion),
    );
    (app, queue)
}
// SAVE-001 / FFI-001: default desktop overlap stays permitted, chooser policy does not bypass dialogs.
#[test]
fn only_reuse_binding_can_save_alongside_pending_source_without_losing_rejected_draft() {
    for policy in [
        SaveDestinationPolicy::ReuseBinding,
        SaveDestinationPolicy::AlwaysChoose,
    ] {
        let (mut app, mut queue) = configured(policy);
        assert!(matches!(
            file(
                &mut app,
                &mut queue,
                FileIntent::Navigate(NavigationIntent::Open)
            ),
            ApplicationEffect::ChooseSource(_)
        ));
        draft(&mut app, &mut queue);
        let before = app.projection();
        let bytes = bytes(&app);
        let outcome = app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::File(FileIntent::Save),
        );
        match policy {
            SaveDestinationPolicy::ReuseBinding => {
                assert!(matches!(
                    outcome.unwrap().effect,
                    Some(ApplicationEffect::SaveQueued(_))
                ));
                assert_eq!(app.projection().editor.interaction, InteractionState::Idle);
            }
            SaveDestinationPolicy::AlwaysChoose => {
                assert_eq!(outcome.unwrap_err().failure, ApplicationFailure::Busy);
                assert_eq!(app.projection(), before);
                assert_eq!(super::bytes(&app), bytes);
                assert_eq!(queue.outstanding(), 0);
            }
        }
    }
}
#[test]
fn always_choose_rejects_second_chooser_and_stale_reply_before_cancelling_draft() {
    let (mut app, mut queue) = configured(SaveDestinationPolicy::AlwaysChoose);
    let ApplicationEffect::ChooseDestination { token, .. } =
        file(&mut app, &mut queue, FileIntent::Save)
    else {
        panic!()
    };
    reply(&mut app, &mut queue, token, EffectDecision::Cancel);
    file(&mut app, &mut queue, FileIntent::Save);
    draft(&mut app, &mut queue);
    let before = app.projection();
    assert_eq!(
        app.file_availability(FileIntent::Save),
        Err(ApplicationFailure::Busy)
    );
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::ResolveEffect(EffectReply {
                token,
                decision: EffectDecision::Destination(DestinationId(8))
            })
        )
        .is_err()
    );
    assert_eq!(app.projection(), before);
    assert_eq!(queue.outstanding(), 0);
}
