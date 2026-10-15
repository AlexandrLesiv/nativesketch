use super::*;
#[test]
fn outcomes_distinguish_settings_selection_identity_and_preview_changes() {
    let mut editor = session(true, true);
    let outcome = execute(&mut editor, EditorCommand::SetBrush(brush(12.0)));
    assert!(outcome.changes.settings);
    assert!(!outcome.changes.document);
    assert!(!outcome.changes.session_replaced);
    let outcome = execute(&mut editor, EditorCommand::SetTool(ToolId::Select));
    assert!(outcome.changes.settings);
    let outcome = execute(&mut editor, EditorCommand::SelectCycle(1));
    assert!(outcome.changes.selection);
    assert!(!outcome.changes.document);
    let outcome = execute(&mut editor, EditorCommand::BeginTransform);
    assert!(outcome.changes.interaction);
    let outcome = execute(
        &mut editor,
        EditorCommand::UpdateTransform {
            scale: 2.0,
            anchor_x: 0.0,
            anchor_y: 0.0,
        },
    );
    assert!(outcome.changes.interaction);
    assert!(!outcome.changes.document);
    execute(&mut editor, EditorCommand::CancelInteraction);
    let outcome = execute(&mut editor, EditorCommand::NewDocument);
    assert!(outcome.changes.session_replaced);
    assert!(outcome.changes.document);
    assert!(outcome.changes.selection);
    assert!(!outcome.changes.settings);
}
#[test]
fn snapshot_is_owned_committed_content_and_scene_identity_tracks_preview() {
    let mut editor = session(true, true);
    let QueryResult::Document(copy) = editor.query(EditorQuery::CommittedSnapshot).unwrap() else {
        panic!()
    };
    let QueryResult::Scene(before) = editor.query(EditorQuery::Scene).unwrap() else {
        panic!()
    };
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
    let QueryResult::Scene(during) = editor.query(EditorQuery::Scene).unwrap() else {
        panic!()
    };
    assert_eq!(during.key.session, before.key.session);
    assert_eq!(during.key.revision, before.key.revision);
    assert_ne!(during.key.generation, before.key.generation);
    assert_eq!(snapshot(&editor), FIXTURE); // An explicit query excludes active and predicted input.
    execute(&mut editor, EditorCommand::CommitStroke(PointerId(42)));
    assert_ne!(snapshot(&editor), FIXTURE);
    assert_eq!(copy.encode().unwrap(), FIXTURE); // Owned worker input remains stable.
}
