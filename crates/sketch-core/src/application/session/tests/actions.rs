use super::*;
fn action(editor: &mut EditorSession, action: EditorAction) -> CommandOutcome {
    editor.action(context(editor), action).unwrap()
}
fn draw(editor: &mut EditorSession) {
    let ctx = begin(editor);
    editor
        .append(
            ctx,
            SampleBatch {
                pointer: PointerId(42),
                samples: &[Input::Actual(sample(1))],
            },
        )
        .unwrap();
    execute(editor, EditorCommand::CommitStroke(PointerId(42)));
}
// INK-001, UNDO-001/002: high-level Undo differs intentionally from primitive Undo.
#[test]
fn active_draft_action_undo_cancels_and_undoes_one_committed_stroke() {
    let mut editor = session(true, false);
    draw(&mut editor);
    let committed = snapshot(&editor);
    let ctx = begin(&mut editor);
    editor
        .append(
            ctx,
            SampleBatch {
                pointer: PointerId(42),
                samples: &[Input::Actual(sample(2)), Input::Predicted(sample(3))],
            },
        )
        .unwrap();
    assert_eq!(
        editor.view_state().availability(ActionId::Undo),
        Availability::Disabled(DisabledReason::InteractionActive)
    );
    assert_eq!(
        editor.view_state().action_availability(EditorAction::Undo),
        Availability::Enabled
    );
    let preview = scene(&editor);
    assert!(editor.execute(ctx, EditorCommand::Undo).is_err());
    assert_eq!(scene(&editor), preview);
    let result = action(&mut editor, EditorAction::Undo);
    assert!(result.changes.document && result.changes.interaction);
    assert_eq!(result.state.interaction, InteractionState::Idle);
    assert_eq!(result.state.document.count, 0);
    action(&mut editor, EditorAction::Redo);
    assert_eq!(snapshot(&editor), committed);
    assert!(!editor.view_state().document.can_redo);
}
#[test]
fn choose_tool_and_cycle_cancel_once_without_history_then_delete_is_one_operation() {
    let mut editor = session(true, true);
    let original = snapshot(&editor);
    begin(&mut editor);
    let outcome = action(&mut editor, EditorAction::ChooseTool(ToolId::Select));
    assert!(outcome.changes.settings && outcome.changes.interaction);
    assert_eq!(snapshot(&editor), original);
    action(
        &mut editor,
        EditorAction::CycleSelection(CycleDirection::Next),
    );
    let first = editor.view_state().document.selected;
    assert_ne!(first, 0);
    action(
        &mut editor,
        EditorAction::CycleSelection(CycleDirection::Previous),
    );
    assert_ne!(editor.view_state().document.selected, 0);
    action(&mut editor, EditorAction::DeleteSelection);
    assert_eq!(editor.view_state().document.selected, 0);
    action(&mut editor, EditorAction::Undo);
    assert_eq!(snapshot(&editor), original);
    assert!(!editor.view_state().document.can_undo);
}
// UNDO-001: committed selection center, never the active transform preview center.
#[test]
fn one_shot_scale_ignores_preview_and_has_exactly_one_undo_redo() {
    let mut editor = session(true, true);
    action(
        &mut editor,
        EditorAction::CycleSelection(CycleDirection::Next),
    );
    let original = snapshot(&editor);
    let QueryResult::SelectedBounds(bounds) = editor.query(EditorQuery::SelectedBounds).unwrap()
    else {
        panic!()
    };
    execute(&mut editor, EditorCommand::BeginTransform);
    execute(
        &mut editor,
        EditorCommand::UpdateTransform {
            scale: 4.0,
            anchor_x: -100.0,
            anchor_y: -200.0,
        },
    );
    action(
        &mut editor,
        EditorAction::ScaleSelection {
            factor: 2.0.try_into().unwrap(),
            anchor: ScaleAnchor::SelectionCenter,
        },
    );
    let scaled = snapshot(&editor);
    let QueryResult::SelectedBounds(after) = editor.query(EditorQuery::SelectedBounds).unwrap()
    else {
        panic!()
    };
    assert!((after.x + after.width / 2.0 - bounds.x - bounds.width / 2.0).abs() < 1e-8);
    assert!((after.width - bounds.width * 2.0).abs() < 1e-8);
    action(&mut editor, EditorAction::Undo);
    assert_eq!(snapshot(&editor), original);
    assert!(!editor.view_state().document.can_undo);
    action(&mut editor, EditorAction::Redo);
    assert_eq!(snapshot(&editor), scaled);
    let revision = editor.view_state().revision;
    action(
        &mut editor,
        EditorAction::ScaleSelection {
            factor: 1.0.try_into().unwrap(),
            anchor: ScaleAnchor::SelectionCenter,
        },
    );
    assert_eq!(editor.view_state().revision, revision);
    action(&mut editor, EditorAction::Undo);
    assert_eq!(snapshot(&editor), original);
}
