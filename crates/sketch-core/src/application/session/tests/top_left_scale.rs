use super::*;
fn action(editor: &mut EditorSession, factor: f64) -> CommandOutcome {
    editor.action(context(editor), scale(factor)).unwrap()
}
fn scale(factor: f64) -> EditorAction {
    EditorAction::ScaleSelection {
        factor: factor.try_into().unwrap(),
        anchor: ScaleAnchor::SelectionTopLeft,
    }
}
fn selected() -> EditorSession {
    let mut editor = session(true, true);
    execute(&mut editor, EditorCommand::SetTool(ToolId::Select));
    execute(&mut editor, EditorCommand::SelectCycle(1));
    editor
}
fn bounds(editor: &EditorSession) -> crate::Bounds {
    let QueryResult::SelectedBounds(bounds) = editor.query(EditorQuery::SelectedBounds).unwrap()
    else {
        panic!()
    };
    bounds
}
fn preview(editor: &mut EditorSession) {
    execute(editor, EditorCommand::BeginTransform);
    execute(
        editor,
        EditorCommand::UpdateTransform {
            scale: 4.0,
            anchor_x: -100.0,
            anchor_y: -200.0,
        },
    );
}
fn preserves(editor: &mut EditorSession, kind: FailureKind) {
    let state = editor.view_state();
    let geometry = scene(editor);
    let bytes = snapshot(editor);
    assert_eq!(
        editor.action(context(editor), scale(2.0)).unwrap_err().kind,
        kind
    );
    assert_eq!(editor.view_state(), state);
    assert_eq!(scene(editor), geometry);
    assert_eq!(snapshot(editor), bytes);
}
// ANDROID-SCALE-001, UNDO-001: anchors already transformed committed bounds, not preview.
#[test]
fn top_left_of_transformed_commit_stays_fixed_with_preview_and_one_undo() {
    let mut editor = selected();
    execute(&mut editor, EditorCommand::BeginTransform);
    execute(
        &mut editor,
        EditorCommand::UpdateTransform {
            scale: 1.5,
            anchor_x: -20.0,
            anchor_y: 30.0,
        },
    );
    execute(&mut editor, EditorCommand::CommitTransform);
    let before = bounds(&editor);
    let original = snapshot(&editor);
    preview(&mut editor);
    let outcome = action(&mut editor, 2.0);
    assert!(outcome.changes.document && outcome.changes.interaction);
    assert_eq!(outcome.state.interaction, InteractionState::Idle);
    let after = bounds(&editor);
    for (actual, expected) in [
        (after.x, before.x),
        (after.y, before.y),
        (after.width, before.width * 2.0),
        (after.height, before.height * 2.0),
    ] {
        assert!((actual - expected).abs() < 1e-8);
    }
    let scaled = snapshot(&editor);
    execute(&mut editor, EditorCommand::Undo);
    assert_eq!(snapshot(&editor), original);
    execute(&mut editor, EditorCommand::Redo);
    assert_eq!(snapshot(&editor), scaled);
}
// UNDO-002: identity cancels preview, adds no revision/history; correlation is not dedup.
#[test]
fn identity_and_repeated_relative_scale_have_defined_history() {
    let mut editor = selected();
    let original = snapshot(&editor);
    let initial = bounds(&editor);
    preview(&mut editor);
    let revision = editor.view_state().revision;
    action(&mut editor, 1.0);
    assert_eq!(editor.view_state().revision, revision);
    assert_eq!(snapshot(&editor), original);
    assert!(!editor.view_state().document.can_undo);
    action(&mut editor, 2.0);
    let once = snapshot(&editor);
    action(&mut editor, 2.0); // same RequestId(17), relative scale applies again
    assert!((bounds(&editor).width - initial.width * 4.0).abs() < 1e-8);
    execute(&mut editor, EditorCommand::Undo);
    assert_eq!(snapshot(&editor), once);
    execute(&mut editor, EditorCommand::Undo);
    assert_eq!(snapshot(&editor), original);
    assert!(!editor.view_state().document.can_undo);
}
// INK-001, UNDO-002: all preconditions precede cancellation, including capacity.
#[test]
fn capacity_and_composed_bounds_rejection_preserve_preview() {
    let mut editor = selected();
    preview(&mut editor);
    editor.editor.history.exhaust_revision_capacity();
    preserves(&mut editor, FailureKind::ResourceLimit);
    action(&mut editor, 1.0); // identity needs no revision capacity
    assert!(!editor.view_state().document.can_undo);
    let mut editor = selected();
    while editor.action(context(&editor), scale(2.0)).is_ok() {}
    let committed = bounds(&editor);
    execute(&mut editor, EditorCommand::BeginTransform);
    execute(
        &mut editor,
        EditorCommand::UpdateTransform {
            scale: 0.5,
            anchor_x: committed.x,
            anchor_y: committed.y,
        },
    );
    preserves(&mut editor, FailureKind::InvalidValue);
}
#[test]
fn top_left_availability_preserves_readonly_and_missing_selection_rules() {
    let mut readonly = session(false, true);
    execute(&mut readonly, EditorCommand::SelectCycle(1));
    assert_eq!(
        readonly.view_state().action_availability(scale(2.0)),
        Availability::Disabled(DisabledReason::ReadOnly)
    );
    preserves(&mut readonly, FailureKind::ReadOnly);
    let mut empty = session(true, false);
    begin(&mut empty);
    assert_eq!(
        empty.view_state().action_availability(scale(2.0)),
        Availability::Disabled(DisabledReason::NothingSelected)
    );
    preserves(&mut empty, FailureKind::InvalidTransition);
}
