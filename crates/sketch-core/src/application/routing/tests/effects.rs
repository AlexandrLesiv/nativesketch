use super::*;
#[test]
fn rejected_effect_and_wrapped_replacement_preserve_draft_and_pending_intent() {
    let mut app = app(true, true);
    let mut queue = queue();
    let ApplicationEffect::ChooseDestination { token, .. } =
        file(&mut app, &mut queue, FileIntent::SaveAs)
    else {
        panic!()
    };
    draft(&mut app, &mut queue);
    let before = app.projection();
    let data = bytes(&app);
    for command in [
        ApplicationCommand::ResolveEffect(EffectReply {
            token,
            decision: EffectDecision::Source(DestinationId(1)),
        }),
        ApplicationCommand::Edit(EditorCommand::NewDocument),
        ApplicationCommand::Edit(EditorCommand::ReplaceDocument(
            ValidatedDocument::decode(FIXTURE).unwrap(),
        )),
    ] {
        assert!(app.dispatch(&mut queue, context(&app), command).is_err());
        assert_eq!(app.projection(), before);
        assert_eq!(bytes(&app), data);
    }
    assert!(matches!(
        reply(&mut app, &mut queue, token, EffectDecision::Cancel),
        ApplicationEffect::Cancelled
    ));
    assert_eq!(
        app.projection().editor.interaction,
        before.editor.interaction
    );
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::ResolveEffect(EffectReply {
                token,
                decision: EffectDecision::Destination(DestinationId(1))
            })
        )
        .is_err()
    );
}
#[test]
fn capability_and_busy_preflight_do_not_cancel_ink_but_accepted_file_intent_does() {
    let mut app = app(true, false);
    let mut queue = queue();
    draft(&mut app, &mut queue);
    let before = app.projection();
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::File(FileIntent::SaveAs)
        )
        .is_err()
    );
    assert_eq!(app.projection(), before);
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::File(FileIntent::Checkpoint(DestinationId(2)))
        )
        .is_err()
    );
    assert_eq!(app.projection(), before);
    let _ = file(
        &mut app,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::Open),
    );
    assert_eq!(app.projection().editor.interaction, InteractionState::Idle);
    assert_eq!(bytes(&app), FIXTURE);
}
#[test]
fn readonly_edit_save_and_recovery_are_denied_but_export_uses_same_issued_route() {
    let mut app = app(false, true);
    let mut queue = queue();
    for intent in [
        FileIntent::SaveAs,
        FileIntent::Navigate(NavigationIntent::Recover(RecoverySourceId(1))),
    ] {
        assert!(
            app.dispatch(&mut queue, context(&app), ApplicationCommand::File(intent))
                .is_err()
        );
    }
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::Edit(EditorCommand::DeleteSelection)
        )
        .is_err()
    );
    let ApplicationEffect::ChooseDestination { token, purpose } =
        file(&mut app, &mut queue, FileIntent::ExportCopy)
    else {
        panic!()
    };
    assert_eq!(purpose, SavePurpose::ExportCopy);
    assert!(matches!(
        reply(
            &mut app,
            &mut queue,
            token,
            EffectDecision::Destination(DestinationId(5))
        ),
        ApplicationEffect::SaveQueued(_)
    ));
    assert!(!app.projection().editor.document.dirty);
}
#[test]
fn input_rejection_does_not_cancel_or_partially_append() {
    let mut app = app(true, true);
    let mut queue = queue();
    draft(&mut app, &mut queue);
    let before = app.projection();
    let sample = Sample {
        id: 3,
        x: 30.0,
        y: 20.0,
        timestamp: 3.0,
        pressure: None,
    };
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::Input(SampleBatch {
                pointer: PointerId(7),
                samples: &[
                    Input::Actual(sample),
                    Input::Actual(Sample {
                        x: f64::NAN,
                        id: 4,
                        ..sample
                    })
                ]
            })
        )
        .is_err()
    );
    assert_eq!(app.projection(), before);
    edit(
        &mut app,
        &mut queue,
        EditorCommand::CommitStroke(PointerId(7)),
    );
    edit(&mut app, &mut queue, EditorCommand::Undo);
    assert_eq!(bytes(&app), FIXTURE);
}
