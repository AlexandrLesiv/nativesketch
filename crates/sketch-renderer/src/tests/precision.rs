use super::fixtures::*;
use crate::{Compositor, RenderError};
use sketch_core::{
    Brush, Input, Sample, SceneCache,
    application::{
        BrushSettings, Capabilities, CommandContext, DeviceKind, EditorCommand, EditorSession,
        InteractionState, PointerId, RequestId, SampleBatch, ToolId,
    },
};
fn editor_stroke(start: f64, end: f64) -> EditorSession {
    let mut editor = EditorSession::new(
        Capabilities {
            edit_document: true,
            export_document: true,
        },
        ToolId::Pen,
        BrushSettings::try_from(Brush {
            width: 4.0,
            rgba: [0.0, 0.0, 0.0, 1.0],
        })
        .unwrap(),
        None,
    )
    .unwrap();
    let state = editor.view_state();
    let mut context = CommandContext {
        session: state.session,
        request: RequestId(1),
        interaction: None,
    };
    let outcome = editor
        .execute(
            context,
            EditorCommand::BeginStroke {
                pointer: PointerId(1),
                device: DeviceKind::Pen,
            },
        )
        .unwrap();
    let InteractionState::Drawing(id) = outcome.state.interaction else {
        panic!()
    };
    context.interaction = Some(id);
    let samples = [start, end]
        .into_iter()
        .enumerate()
        .map(|(index, x)| {
            Input::Actual(Sample {
                x,
                y: 20.0,
                timestamp: index as f64,
                pressure: None,
                id: index as u64 + 1,
            })
        })
        .collect::<Vec<_>>();
    editor
        .append(
            context,
            SampleBatch {
                pointer: PointerId(1),
                samples: &samples,
            },
        )
        .unwrap();
    editor
        .execute(context, EditorCommand::CommitStroke(PointerId(1)))
        .unwrap();
    editor
}
#[test]
fn actual_editor_long_stroke_preserves_far_endpoint_in_positive_and_negative_viewports() {
    for (start, end, offset) in [
        (0.0, 999_999_010.0, -999_999_000.0),
        (0.0, -999_999_010.0, 999_999_020.0),
    ] {
        let editor = editor_stroke(start, end);
        let (_, mut request) = scene(vec![]);
        let state = editor.view_state();
        request.session = state.session.get();
        request.revision = state.revision.0;
        request.scene_generation = state.scene_generation.get();
        request.offset_x = offset;
        let scene = editor
            .render_scene(&mut SceneCache::default(), request.viewport())
            .unwrap();
        let (cached, _) = render(&mut Compositor::default(), &scene, &request, true);
        let (reference, _) = render(&mut Compositor::default(), &scene, &request, false);
        equivalent(&cached, &reference, "long-endpoint");
        assert_eq!(pixel(&cached, 10, 20), &[0, 0, 0, 255]);
        let outside = if end > 0.0 { 13 } else { 7 };
        assert_eq!(pixel(&cached, outside, 20), &[255, 255, 255, 255]);
    }
}
#[test]
fn enclosing_giant_curved_contour_keeps_fill_and_single_alpha() {
    let editor = editor_stroke(0.0, 0.0);
    let (_, mut request) = scene(vec![]);
    let state = editor.view_state();
    request.session = state.session.get();
    request.revision = state.revision.0;
    request.scene_generation = state.scene_generation.get();
    let mut scene = editor
        .render_scene(&mut SceneCache::default(), request.viewport())
        .unwrap();
    let stroke = &mut std::sync::Arc::make_mut(&mut scene.committed)[0];
    stroke.transform.scale = 1_000_000.0;
    stroke.transform.tx = 32.0;
    stroke.transform.ty = -20_000_000.0 + 32.0;
    let mut geometry = sketch_core::StrokeGeometry {
        commands: stroke.geometry.commands.clone(),
        bounds: stroke.geometry.bounds,
        origin: stroke.geometry.origin,
    };
    let mut commands = geometry.commands.to_vec();
    for command in &mut commands {
        command.rgba = [1.0, 0.0, 0.0, 0.5];
    }
    geometry.commands = commands.into();
    stroke.geometry = std::sync::Arc::new(geometry);
    let (pixels, _) = render(&mut Compositor::default(), &scene, &request, true);
    assert_eq!(pixel(&pixels, 1, 1), &[255, 127, 127, 255]);
    assert_eq!(pixel(&pixels, 32, 32), &[255, 127, 127, 255]);
    assert_eq!(pixel(&pixels, 63, 63), &[255, 127, 127, 255]);
    // Move the giant circular edge through the viewport; its rightmost point is x=10.
    std::sync::Arc::make_mut(&mut scene.committed)[0]
        .transform
        .tx = -2_000_000.0 + 10.0;
    scene.key.revision.0 += 1;
    request.revision += 1;
    let (grazing, _) = render(&mut Compositor::default(), &scene, &request, true);
    assert_eq!(pixel(&grazing, 9, 32), &[255, 127, 127, 255]);
    assert_eq!(pixel(&grazing, 11, 32), &[255, 255, 255, 255]);
}
#[test]
fn clipping_budget_exhaustion_is_explicit() {
    let mut points = vec![[0.0, 0.0]; crate::clip::MAX_POINTS];
    assert_eq!(
        crate::clip::push(&mut points, [1.0, 1.0]),
        Err(RenderError::ResourceLimit)
    );
}
