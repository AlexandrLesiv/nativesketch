use super::*;
fn preserve(
    editor: &mut EditorSession,
    context: CommandContext,
    action: EditorAction,
    kind: FailureKind,
) {
    let state = editor.view_state();
    let preview = scene(editor);
    let bytes = snapshot(editor);
    assert_eq!(editor.action(context, action).unwrap_err().kind, kind);
    assert_eq!(editor.view_state(), state);
    assert_eq!(scene(editor), preview);
    assert_eq!(snapshot(editor), bytes);
}
// INK-001, ASYNC-001: unavailable intents never consume a draft or prediction.
#[test]
fn unavailable_history_selection_and_stale_context_preserve_draft() {
    let mut editor = session(true, false);
    let ctx = begin(&mut editor);
    editor
        .append(
            ctx,
            SampleBatch {
                pointer: PointerId(42),
                samples: &[Input::Actual(sample(1)), Input::Predicted(sample(2))],
            },
        )
        .unwrap();
    for action in [
        EditorAction::Undo,
        EditorAction::Redo,
        EditorAction::DeleteSelection,
        EditorAction::ScaleSelection {
            factor: 2.0.try_into().unwrap(),
            anchor: ScaleAnchor::SelectionCenter,
        },
    ] {
        preserve(&mut editor, ctx, action, FailureKind::InvalidTransition);
    }
    let stale = CommandContext {
        interaction: Some(InteractionId(u64::MAX)),
        ..ctx
    };
    preserve(
        &mut editor,
        stale,
        EditorAction::ChooseTool(ToolId::Select),
        FailureKind::StaleInteraction,
    );
    let other = context(&session(true, false));
    preserve(
        &mut editor,
        other,
        EditorAction::ChooseTool(ToolId::Select),
        FailureKind::StaleSession,
    );
}
#[test]
fn invalid_scale_and_composed_overflow_preserve_active_transform() {
    for value in [0.0, -1.0, 100.0001, f64::NAN, f64::INFINITY] {
        assert_eq!(ScaleFactor::try_from(value), Err(FailureKind::InvalidValue));
    }
    let mut editor = session(true, true);
    editor
        .action(
            context(&editor),
            EditorAction::CycleSelection(CycleDirection::Next),
        )
        .unwrap();
    let scale = EditorAction::ScaleSelection {
        factor: 100.0.try_into().unwrap(),
        anchor: ScaleAnchor::SelectionCenter,
    };
    while editor.action(context(&editor), scale).is_ok() {}
    execute(&mut editor, EditorCommand::BeginTransform);
    let ctx = context(&editor);
    preserve(&mut editor, ctx, scale, FailureKind::InvalidValue);
}
#[test]
fn exhausted_generation_does_not_cancel_or_change_settings() {
    let mut editor = session(true, true);
    let ctx = begin(&mut editor);
    editor.generation = SceneGeneration(u64::MAX);
    preserve(
        &mut editor,
        ctx,
        EditorAction::ChooseTool(ToolId::Select),
        FailureKind::ResourceLimit,
    );
    preserve(
        &mut editor,
        ctx,
        EditorAction::CycleSelection(CycleDirection::Next),
        FailureKind::ResourceLimit,
    );
}
// VIEW-001: read-only document mutation is rejected even without native controls.
#[test]
fn readonly_compound_mutations_are_denied() {
    let mut editor = session(false, true);
    execute(&mut editor, EditorCommand::SelectCycle(1));
    let ctx = context(&editor);
    for action in [
        EditorAction::ChooseTool(ToolId::Pen),
        EditorAction::Undo,
        EditorAction::Redo,
        EditorAction::DeleteSelection,
        EditorAction::ScaleSelection {
            factor: 2.0.try_into().unwrap(),
            anchor: ScaleAnchor::SelectionCenter,
        },
    ] {
        preserve(&mut editor, ctx, action, FailureKind::ReadOnly);
    }
}

// Empty-document selection controls stay unavailable, including during the first draft.
#[test]
fn empty_cycle_in_both_directions_preserves_draft_and_predictions() {
    let mut editor = session(true, false);
    let ctx = begin(&mut editor);
    editor
        .append(
            ctx,
            SampleBatch {
                pointer: PointerId(42),
                samples: &[Input::Actual(sample(1)), Input::Predicted(sample(2))],
            },
        )
        .unwrap();
    for direction in [CycleDirection::Previous, CycleDirection::Next] {
        let action = EditorAction::CycleSelection(direction);
        assert_eq!(
            editor.view_state().action_availability(action),
            Availability::Disabled(DisabledReason::NothingSelected)
        );
        preserve(&mut editor, ctx, action, FailureKind::InvalidTransition);
    }
}
