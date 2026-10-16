use super::*;
#[test]
fn replacement_rotates_session_preserves_settings_and_rejects_old_contexts() {
    let mut editor = session(true, true);
    execute(&mut editor, EditorCommand::SetBrush(brush(17.0)));
    execute(&mut editor, EditorCommand::SetTool(ToolId::Select));
    execute(&mut editor, EditorCommand::SelectCycle(1));
    let old = context(&editor);
    execute(&mut editor, EditorCommand::NewDocument);
    let state = editor.view_state();
    assert_ne!(state.session, old.session);
    assert_eq!(state.tool, ToolId::Select);
    assert_eq!(state.brush, brush(17.0));
    assert_eq!(state.document.count, 0);
    assert!(!state.document.dirty);
    assert!(!state.document.can_undo);
    assert_eq!(
        editor
            .execute(old, EditorCommand::SelectCycle(1))
            .unwrap_err()
            .kind,
        FailureKind::StaleSession
    );
    let current = context(&editor);
    execute(
        &mut editor,
        EditorCommand::ReplaceDocument(ValidatedDocument::decode(FIXTURE).unwrap()),
    );
    assert_ne!(editor.view_state().session, current.session);
    assert_eq!(snapshot(&editor), FIXTURE);
    assert_eq!(editor.view_state().brush, brush(17.0));
    assert_eq!(editor.view_state().tool, ToolId::Select);
    assert!(!editor.view_state().document.can_undo);
    assert_eq!(
        editor
            .execute(current, EditorCommand::CancelInteraction)
            .unwrap_err()
            .kind,
        FailureKind::StaleSession
    );
    let before = editor.view_state();
    assert!(ValidatedDocument::decode(b"corrupt").is_err());
    assert_eq!(editor.view_state(), before);
}
#[test]
fn distinct_instances_with_same_revision_reject_each_others_context() {
    let first = session(true, false);
    let mut second = session(true, false);
    assert_eq!(first.view_state().revision, second.view_state().revision);
    assert_ne!(first.view_state().session, second.view_state().session);
    assert_eq!(
        second
            .execute(context(&first), EditorCommand::NewDocument)
            .unwrap_err()
            .kind,
        FailureKind::StaleSession
    );
    assert_eq!(second.view_state().revision, Revision(0));
}
#[test]
fn captured_brush_and_tool_rules_are_additive_without_hidden_cancellation() {
    let mut editor = session(true, false);
    let token = begin(&mut editor);
    editor
        .append(
            token,
            SampleBatch {
                pointer: PointerId(42),
                samples: &[Input::Actual(sample(1))],
            },
        )
        .unwrap();
    execute(&mut editor, EditorCommand::SetBrush(brush(19.0)));
    assert_eq!(
        editor
            .execute(token, EditorCommand::SetTool(ToolId::Select))
            .unwrap_err()
            .kind,
        FailureKind::InvalidTransition
    );
    editor
        .execute(token, EditorCommand::CommitStroke(PointerId(42)))
        .unwrap();
    let token = begin(&mut editor);
    editor
        .append(
            token,
            SampleBatch {
                pointer: PointerId(42),
                samples: &[Input::Actual(sample(1))],
            },
        )
        .unwrap();
    editor
        .execute(token, EditorCommand::CommitStroke(PointerId(42)))
        .unwrap();
    let QueryResult::Document(snapshot) = editor.query(EditorQuery::CommittedSnapshot).unwrap()
    else {
        panic!()
    };
    assert_eq!(snapshot.document().strokes[0].brush.width, 6.0);
    assert_eq!(snapshot.document().strokes[1].brush.width, 19.0);
    execute(&mut editor, EditorCommand::Undo);
    execute(&mut editor, EditorCommand::Undo);
    assert!(!editor.view_state().document.can_undo);
    execute(&mut editor, EditorCommand::SetTool(ToolId::Select));
    assert_eq!(
        editor
            .execute(
                context(&editor),
                EditorCommand::BeginStroke {
                    pointer: PointerId(42),
                    device: DeviceKind::Pen
                }
            )
            .unwrap_err()
            .kind,
        FailureKind::InvalidTransition
    );
    assert_eq!(editor.view_state().interaction, InteractionState::Idle);
    let mut legacy = Editor::new();
    legacy.begin(42, 1, brush(6.0).brush()).unwrap(); // Legacy has no new tool policy.
}
#[test]
fn brush_validation_and_request_correlation_are_not_permissions_or_deduplication() {
    assert!(
        BrushSettings::try_from(Brush {
            width: 0.0,
            rgba: [0.0; 4]
        })
        .is_err()
    );
    assert!(
        BrushSettings::try_from(Brush {
            width: 1.0,
            rgba: [f64::NAN; 4]
        })
        .is_err()
    );
    let mut editor = session(true, true);
    let ctx = context(&editor);
    editor
        .execute(ctx, EditorCommand::SetBrush(brush(7.0)))
        .unwrap();
    editor
        .execute(ctx, EditorCommand::SetBrush(brush(9.0)))
        .unwrap();
    assert_eq!(editor.view_state().brush, brush(9.0));
    assert!(!editor.view_state().document.can_undo);
    let mut viewer = session(false, false);
    let ctx = CommandContext {
        request: RequestId(u64::MAX),
        ..context(&viewer)
    };
    let error = viewer.execute(ctx, EditorCommand::NewDocument).unwrap_err();
    assert_eq!(error.kind, FailureKind::ReadOnly);
    assert_eq!(error.context, Some(ctx));
}
