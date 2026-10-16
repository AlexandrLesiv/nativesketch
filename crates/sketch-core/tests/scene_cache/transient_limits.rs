use super::support::*;

fn limited() -> SceneCache {
    SceneCache::new(SceneLimits {
        retained_bytes: 3000,
        prepared_bytes: 3000,
    })
}

#[test]
fn oversized_live_geometry_preserves_committed_root_and_cancellable_transaction() {
    let mut editor = editor();
    stroke(&mut editor, -10.0);
    let saved = bytes(&editor);
    let mut cache = limited();
    let root = editor.render_scene(&mut cache, viewport()).unwrap();
    begin(&mut editor);
    let samples: Vec<_> = (0..20)
        .map(|id| Input::Actual(sample(id, id as f64, 30.0)))
        .collect();
    append(&mut editor, &samples);
    let state = editor.view_state();
    assert!(matches!(
        editor.render_scene(&mut cache, viewport()),
        Err(SceneError::ResourceLimit)
    ));
    assert_eq!(editor.view_state(), state);
    assert_eq!(bytes(&editor), saved);
    execute(&mut editor, EditorCommand::CancelInteraction);
    let restored = editor.render_scene(&mut cache, viewport()).unwrap();
    assert!(Arc::ptr_eq(&root.committed, &restored.committed));
    equivalent(&commands(&root), &commands(&restored));
    execute(&mut editor, EditorCommand::Undo);
    assert!(
        editor
            .render_scene(&mut cache, viewport())
            .unwrap()
            .committed
            .is_empty()
    );
}

#[test]
fn entering_transform_respects_total_scene_limit_without_committing_preview() {
    let mut editor = editor();
    stroke(&mut editor, 10.0);
    stroke(&mut editor, 500.0);
    let saved = bytes(&editor);
    let view = RenderViewport {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
        margin: 1.0,
    };
    let mut cache = limited();
    let root = editor.render_scene(&mut cache, view).unwrap();
    assert_eq!(root.committed.len(), 1);
    execute(&mut editor, EditorCommand::SetTool(ToolId::Select));
    execute(
        &mut editor,
        EditorCommand::SelectAt {
            x: 500.0,
            y: 20.0,
            tolerance: 1.0,
        },
    );
    execute(&mut editor, EditorCommand::BeginTransform);
    execute(
        &mut editor,
        EditorCommand::UpdateTransform {
            scale: 0.1,
            anchor_x: 0.0,
            anchor_y: 0.0,
        },
    );
    let state = editor.view_state();
    assert!(matches!(
        editor.render_scene(&mut cache, view),
        Err(SceneError::ResourceLimit)
    ));
    assert_eq!(editor.view_state(), state);
    assert_eq!(bytes(&editor), saved);
    let unrestricted = editor
        .render_scene(&mut SceneCache::default(), view)
        .unwrap();
    assert!(
        unrestricted
            .transform_preview
            .unwrap()
            .replacement
            .is_some()
    );
    execute(&mut editor, EditorCommand::CancelInteraction);
    let restored = editor.render_scene(&mut cache, view).unwrap();
    assert!(Arc::ptr_eq(&root.committed, &restored.committed));
    assert_eq!(bytes(&editor), saved);
}
