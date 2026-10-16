use super::*;
use sketch_core::{
    Bounds, PathCommand, RenderScene, RenderViewport, SceneStroke, StrokeGeometry, Transform,
    TransformPreview,
};
use std::sync::Arc;
#[test]
fn full_scene_and_paged_paths_preserve_precision_and_optional_zeroes() {
    let h = 0xf123456789abcd00;
    let request = 0xfedcba9876543210;
    let mut a = app();
    let id = session(&mut a);
    let handle = a.session(id).unwrap();
    let p = a.app.projection(handle).unwrap().editor;
    let commands: Vec<_> = [
        (0, [1e9 + 0.125, 2.0, 0.0, 0.0, 0.0, 0.0]),
        (1, [1e9 + 0.25, 3.0, 0.0, 0.0, 0.0, 0.0]),
        (2, [1e9 + 0.375, 4.0, 1e9 + 0.5, 5.0, 1e9 + 0.625, 6.0]),
        (3, [0.0; 6]),
    ]
    .into_iter()
    .map(|(verb, points)| PathCommand {
        stroke_id: h + 64,
        rgba: [0.25, 0.5, 0.75, 0.5],
        verb,
        points,
    })
    .collect();
    let geometry = Arc::new(StrokeGeometry {
        commands: commands.into(),
        bounds: Bounds {
            x: 1e9,
            y: 2.0,
            width: 5.0,
            height: 6.0,
        },
        origin: [1e9, 2.0],
    });
    let stroke = SceneStroke {
        stroke_id: h + 64,
        order: 7,
        geometry: geometry.clone(),
        transform: Transform {
            scale: 2.0,
            tx: 3.0,
            ty: 4.0,
        },
    };
    let live = SceneStroke {
        stroke_id: h + 80,
        order: 8,
        geometry,
        transform: Transform::default(),
    };
    let scene = RenderScene {
        key: SceneKey {
            session: p.session,
            revision: Revision(23),
            generation: p.scene_generation,
        },
        committed: vec![stroke].into(),
        live: Some(live),
        transform_preview: Some(TransformPreview {
            stroke_id: h + 64,
            order: 7,
            replacement: None,
        }),
    };
    let viewport = RenderViewport {
        x: -100.0,
        y: 200.0,
        width: 1024.0,
        height: 768.0,
        margin: 16.0,
    };
    a.scenes.publish(handle, viewport, scene, h).unwrap();
    let mut reply = Reply::reserve(&a.budget, 40, request, 4096).unwrap();
    a.scenes.info(&mut reply, h, false).unwrap();
    let mut actual = reply.finish().bytes().to_vec();
    assert_eq!(u64_at(&actual, 80), p.session.get());
    actual[80..88].copy_from_slice(&(h + 32).to_le_bytes());
    assert_eq!(actual, fixture("response.scene"));
    let mut reply = Reply::reserve(&a.budget, 43, request, 4096).unwrap();
    a.scenes.commands(&mut reply, h, h + 1, 0, 4).unwrap();
    assert_eq!(reply.finish().bytes(), fixture("response.paths"));
    let mut first = Reply::reserve(&a.budget, 43, request, 4096).unwrap();
    a.scenes.commands(&mut first, h, h + 1, 0, 2).unwrap();
    let first = first.finish();
    let mut second = Reply::reserve(&a.budget, 43, request, 4096).unwrap();
    a.scenes.commands(&mut second, h, h + 1, 2, 2).unwrap();
    let second = second.finish();
    let joined = [&first.bytes()[80..], &second.bytes()[80..]].concat();
    assert_eq!(joined, fixture("response.paths")[80..]);
}
