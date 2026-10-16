use super::support::*;

#[test]
fn preview_keeps_original_order_and_shares_geometry_then_cancel_restores_content() {
    let mut editor = editor();
    stroke(&mut editor, 10.0);
    stroke(&mut editor, 15.0);
    let mut cache = SceneCache::default();
    let original = editor.render_scene(&mut cache, viewport()).unwrap();
    let saved = bytes(&editor);
    execute(&mut editor, EditorCommand::SetTool(ToolId::Select));
    execute(&mut editor, EditorCommand::SelectCycle(1));
    execute(&mut editor, EditorCommand::BeginTransform);
    execute(
        &mut editor,
        EditorCommand::UpdateTransform {
            scale: 2.0,
            anchor_x: 0.0,
            anchor_y: 0.0,
        },
    );
    let preview = editor.render_scene(&mut cache, viewport()).unwrap();
    assert!(Arc::ptr_eq(&original.committed, &preview.committed));
    let replacement = preview
        .transform_preview
        .as_ref()
        .unwrap()
        .replacement
        .as_ref()
        .unwrap();
    assert_eq!(replacement.order, 0);
    assert!(Arc::ptr_eq(
        &replacement.geometry,
        &original.committed[0].geometry
    ));
    assert_eq!(cache.stats().geometry_builds, 2);
    reference(&editor, &preview);
    assert_eq!(bytes(&editor), saved);
    execute(&mut editor, EditorCommand::CancelInteraction);
    let cancelled = editor.render_scene(&mut cache, viewport()).unwrap();
    assert!(cancelled.transform_preview.is_none());
    equivalent(&commands(&original), &commands(&cancelled));
}

#[test]
fn antialias_margin_predictions_and_transform_crossing_are_not_culled_incorrectly() {
    let mut editor = editor();
    stroke(&mut editor, 10.0);
    let mut cache = SceneCache::default();
    let view = RenderViewport {
        x: 22.0,
        y: 0.0,
        width: 20.0,
        height: 50.0,
        margin: 0.0,
    };
    assert!(
        editor
            .render_scene(&mut cache, view)
            .unwrap()
            .committed
            .is_empty()
    );
    assert_eq!(
        editor
            .render_scene(
                &mut cache,
                RenderViewport {
                    margin: 1.0,
                    ..view
                }
            )
            .unwrap()
            .committed
            .len(),
        1
    );
    begin(&mut editor);
    append(
        &mut editor,
        &[
            Input::Actual(sample(1, 0.0, 20.0)),
            Input::Predicted(sample(2, 30.0, 20.0)),
        ],
    );
    let live = editor.render_scene(&mut cache, view).unwrap();
    assert!(live.live.is_some());
    execute(&mut editor, EditorCommand::CancelInteraction);
    execute(&mut editor, EditorCommand::SetTool(ToolId::Select));
    execute(&mut editor, EditorCommand::SelectCycle(1));
    execute(&mut editor, EditorCommand::BeginTransform);
    execute(
        &mut editor,
        EditorCommand::UpdateTransform {
            scale: 2.0,
            anchor_x: 0.0,
            anchor_y: 20.0,
        },
    );
    let entering = editor.render_scene(&mut cache, view).unwrap();
    assert!(entering.committed.is_empty());
    assert!(entering.transform_preview.unwrap().replacement.is_some());
    execute(
        &mut editor,
        EditorCommand::UpdateTransform {
            scale: 10.0,
            anchor_x: 0.0,
            anchor_y: 20.0,
        },
    );
    assert!(
        editor
            .render_scene(&mut cache, view)
            .unwrap()
            .transform_preview
            .unwrap()
            .replacement
            .is_none()
    );
}

#[test]
fn large_coordinate_geometry_keeps_small_local_differences_in_f64() {
    let mut editor = editor();
    let x = 999_999_000.0;
    stroke(&mut editor, x);
    let view = RenderViewport {
        x: x - 5.0,
        y: 0.0,
        width: 30.0,
        height: 50.0,
        margin: 1.0,
    };
    let scene = editor
        .render_scene(&mut SceneCache::default(), view)
        .unwrap();
    let geometry = &scene.committed[0].geometry;
    reference(&editor, &scene);
    let local = geometry.commands[0].points[0] - geometry.origin[0];
    assert!(local > 3.0 && local < 3.1);
    assert!(geometry.bounds.width > 13.0 && geometry.bounds.width < 13.1);
    assert!(geometry.commands[0].points[0] as f32 == geometry.origin[0] as f32);
}
