use super::*;
use sketch_core::{
    Bounds, PathCommand, RenderScene, RenderViewport, SceneStroke, StrokeGeometry, Transform,
};
use std::sync::Arc;
pub(super) fn leased(a: &mut WireApp, id: u64) -> u64 {
    let handle = a.session(id).unwrap();
    let state = a.app.projection(handle).unwrap().editor;
    let commands = (0..300)
        .flat_map(|_| {
            [
                (0, [0.0, 18.0]),
                (1, [1e9, 18.0]),
                (1, [1e9, 22.0]),
                (1, [0.0, 22.0]),
                (3, [0.0, 0.0]),
            ]
        })
        .map(|(verb, p)| PathCommand {
            stroke_id: 0xfedcba9876543210,
            rgba: [1.0, 0.0, 0.0, 0.5],
            verb,
            points: [p[0], p[1], 0.0, 0.0, 0.0, 0.0],
        })
        .collect::<Vec<_>>();
    let stroke = SceneStroke {
        stroke_id: 0xfedcba9876543210,
        order: 0,
        transform: Transform::default(),
        geometry: Arc::new(StrokeGeometry {
            commands: commands.into(),
            origin: [0.0, 18.0],
            bounds: Bounds {
                x: 0.0,
                y: 18.0,
                width: 1e9,
                height: 4.0,
            },
        }),
    };
    let scene = RenderScene {
        key: SceneKey {
            session: state.session,
            revision: state.revision,
            generation: state.scene_generation,
        },
        committed: vec![stroke.clone()].into(),
        live: Some(stroke.clone()),
        transform_preview: Some(sketch_core::TransformPreview {
            stroke_id: stroke.stroke_id,
            order: 0,
            replacement: Some(stroke),
        }),
    };
    let scene_id = crate::identity::reserve(100).unwrap();
    a.scenes
        .publish(
            handle,
            RenderViewport {
                x: 1e9 - 10.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
                margin: 2.0,
            },
            scene,
            scene_id,
        )
        .unwrap();
    scene_id
}
pub(super) fn prepare(
    a: &mut WireApp,
    scene: u64,
    selector: u32,
    index: u32,
) -> Result<Reply, WireError> {
    let mut b = words(&[scene]);
    for v in [selector, index, 100, 100] {
        b.extend(v.to_le_bytes());
    }
    a.call(&packet(45, &b))
}
pub(super) fn page(a: &mut WireApp, id: u64, first: u32, limit: u32) -> Result<Reply, WireError> {
    let mut b = words(&[id]);
    b.extend(first.to_le_bytes());
    b.extend(limit.to_le_bytes());
    a.call(&packet(46, &b))
}
#[test]
fn prepared_pages_are_complete_independent_and_released_exactly_once() {
    let mut a = app();
    let id = session(&mut a);
    let scene = leased(&mut a, id);
    let r = prepare(&mut a, scene, 0, 0).unwrap();
    let prepared = u64_at(r.bytes(), 80);
    let count = u64_at(r.bytes(), 120);
    assert_eq!(r.bytes().len(), 136);
    assert_eq!(u32_at(r.bytes(), 72), 1);
    assert_eq!(u64_at(r.bytes(), 88), scene);
    assert_eq!(u64_at(r.bytes(), 104), 0xfedcba9876543210);
    assert_eq!(count, 1500);
    drop(r);
    assert_eq!(prepare(&mut a, scene, 0, 0).unwrap_err().status, 9);
    drop(a.call(&packet(44, &words(&[scene]))).unwrap());
    let mut all = Vec::new();
    for first in [0, 744, 1488] {
        let r = page(&mut a, prepared, first, 1000).unwrap();
        assert!(r.bytes().len() <= 65616);
        all.extend_from_slice(&r.bytes()[80..]);
    }
    assert_eq!(all.len(), count as usize * 88);
    assert_eq!(
        u32_at(page(&mut a, prepared, 1500, 1).unwrap().bytes(), 72),
        0
    );
    assert_eq!(page(&mut a, prepared, 1501, 1).unwrap_err().status, 1);
    assert_eq!(u32_at(page(&mut a, prepared, 0, 0).unwrap().bytes(), 72), 0);
    drop(a.call(&packet(2, &words(&[id]))).unwrap());
    drain(&mut a);
    assert!(!a.shutdown_ready());
    drop(a.call(&packet(47, &words(&[prepared]))).unwrap());
    assert!(a.shutdown_ready());
    assert_eq!(
        a.call(&packet(47, &words(&[prepared]))).unwrap_err().status,
        7
    );
    assert_eq!(page(&mut a, prepared, 0, 1).unwrap_err().status, 7);
}
#[test]
fn invalid_selection_capacity_foreign_and_duplicate_calls_preserve_owners() {
    let mut a = app();
    let id = session(&mut a);
    let scene = leased(&mut a, id);
    for (selector, index) in [(3, 0), (1, 1), (2, 1), (0, 1)] {
        assert_eq!(
            prepare(&mut a, scene, selector, index).unwrap_err().status,
            1
        );
    }
    let limit = a.scenes.limits.scene_bytes;
    a.scenes.limits.scene_bytes = a.scenes.bytes + 256;
    assert_eq!(prepare(&mut a, scene, 0, 0).unwrap_err().status, 3);
    assert!(a.scenes.prepared.is_none());
    a.scenes.limits.scene_bytes = limit;
    let mut held = Vec::new();
    for _ in 0..a.scenes.limits.buffers {
        held.push(a.call(&packet(5, &[])).unwrap());
    }
    assert_eq!(prepare(&mut a, scene, 0, 0).unwrap_err().status, 3);
    assert!(a.scenes.prepared.is_none());
    drop(held);
    let r = prepare(&mut a, scene, 1, 0).unwrap();
    let prepared = u64_at(r.bytes(), 80);
    drop(r);
    let mut b = app();
    assert_eq!(page(&mut b, prepared, 0, 1).unwrap_err().status, 7);
    assert_eq!(
        b.call(&packet(47, &words(&[prepared]))).unwrap_err().status,
        7
    );
    assert_eq!(prepare(&mut a, scene, 0, 0).unwrap_err().status, 9);
    drop(a.call(&packet(47, &words(&[prepared]))).unwrap());
    let next = prepare(&mut a, scene, 2, 0).unwrap();
    assert_ne!(prepared, u64_at(next.bytes(), 80));
}
