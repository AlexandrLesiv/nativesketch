use super::support::*;

#[test]
fn committed_root_is_shared_through_predictions_and_commit_has_no_duplicate_ink() {
    let mut editor = editor();
    stroke(&mut editor, 10.0);
    let mut cache = SceneCache::default();
    let initial = editor.render_scene(&mut cache, viewport()).unwrap();
    let saved = bytes(&editor);
    begin(&mut editor);
    append(
        &mut editor,
        &[
            Input::Actual(sample(1, 50.0, 20.0)),
            Input::Predicted(sample(2, 60.0, 20.0)),
        ],
    );
    let live = editor.render_scene(&mut cache, viewport()).unwrap();
    assert!(Arc::ptr_eq(&initial.committed, &live.committed));
    assert_eq!(live.live.as_ref().unwrap().geometry.commands.len(), 17);
    reference(&editor, &live);
    assert_eq!(bytes(&editor), saved);
    append(&mut editor, &[Input::Actual(sample(2, 55.0, 22.0))]);
    let replaced = editor.render_scene(&mut cache, viewport()).unwrap();
    assert!(Arc::ptr_eq(&initial.committed, &replaced.committed));
    reference(&editor, &replaced);
    execute(&mut editor, EditorCommand::CommitStroke(PointerId(1)));
    let committed = editor.render_scene(&mut cache, viewport()).unwrap();
    assert!(committed.live.is_none());
    assert_eq!(committed.committed.len(), 2);
    equivalent(&commands(&replaced), &commands(&committed));
    execute(&mut editor, EditorCommand::Undo);
    reference(
        &editor,
        &editor.render_scene(&mut cache, viewport()).unwrap(),
    );
    assert_eq!(bytes(&editor), saved);
    execute(&mut editor, EditorCommand::Redo);
    equivalent(
        &commands(&committed),
        &commands(&editor.render_scene(&mut cache, viewport()).unwrap()),
    );
    assert_eq!(cache.stats().committed_builds, 4);
    assert_eq!(cache.stats().committed_reuses, 2);
}

#[test]
fn cancellation_id_reuse_new_session_and_import_never_reuse_wrong_geometry() {
    let mut editor = editor();
    let mut cache = SceneCache::default();
    begin(&mut editor);
    append(&mut editor, &[Input::Actual(sample(1, 500.0, 20.0))]);
    let cancelled = editor.render_scene(&mut cache, viewport()).unwrap();
    execute(&mut editor, EditorCommand::CancelInteraction);
    stroke(&mut editor, 10.0);
    let first = editor.render_scene(&mut cache, viewport()).unwrap();
    assert_eq!(
        cancelled.live.unwrap().stroke_id,
        first.committed[0].stroke_id
    );
    reference(&editor, &first);
    let saved = bytes(&editor);
    execute(&mut editor, EditorCommand::NewDocument);
    stroke(&mut editor, 100.0);
    let second = editor.render_scene(&mut cache, viewport()).unwrap();
    assert_ne!(first.key.session, second.key.session);
    assert_eq!(first.committed[0].stroke_id, second.committed[0].stroke_id);
    assert!(!Arc::ptr_eq(
        &first.committed[0].geometry,
        &second.committed[0].geometry
    ));
    reference(&editor, &second);
    execute(
        &mut editor,
        EditorCommand::ReplaceDocument(ValidatedDocument::decode(&saved).unwrap()),
    );
    let imported = editor.render_scene(&mut cache, viewport()).unwrap();
    assert_ne!(imported.key.session, second.key.session);
    equivalent(&commands(&first), &commands(&imported));
    execute(&mut editor, EditorCommand::SetTool(ToolId::Select));
    execute(&mut editor, EditorCommand::SelectCycle(1));
    execute(&mut editor, EditorCommand::DeleteSelection);
    assert!(
        editor
            .render_scene(&mut cache, viewport())
            .unwrap()
            .committed
            .is_empty()
    );
    execute(&mut editor, EditorCommand::Undo);
    reference(
        &editor,
        &editor.render_scene(&mut cache, viewport()).unwrap(),
    );
}

#[test]
fn committed_transform_undo_redo_reuse_geometry_but_update_presentation_metadata() {
    let mut editor = editor();
    stroke(&mut editor, 10.0);
    let mut cache = SceneCache::default();
    let original = editor.render_scene(&mut cache, viewport()).unwrap();
    execute(&mut editor, EditorCommand::SetTool(ToolId::Select));
    execute(&mut editor, EditorCommand::SelectCycle(1));
    execute(&mut editor, EditorCommand::BeginTransform);
    execute(
        &mut editor,
        EditorCommand::UpdateTransform {
            scale: 3.0,
            anchor_x: 7.0,
            anchor_y: 9.0,
        },
    );
    execute(&mut editor, EditorCommand::CommitTransform);
    let changed = editor.render_scene(&mut cache, viewport()).unwrap();
    assert!(Arc::ptr_eq(
        &original.committed[0].geometry,
        &changed.committed[0].geometry
    ));
    assert_ne!(
        original.committed[0].transform,
        changed.committed[0].transform
    );
    reference(&editor, &changed);
    execute(&mut editor, EditorCommand::Undo);
    equivalent(
        &commands(&original),
        &commands(&editor.render_scene(&mut cache, viewport()).unwrap()),
    );
    execute(&mut editor, EditorCommand::Redo);
    equivalent(
        &commands(&changed),
        &commands(&editor.render_scene(&mut cache, viewport()).unwrap()),
    );
    assert_eq!(cache.stats().geometry_builds, 1);
}
