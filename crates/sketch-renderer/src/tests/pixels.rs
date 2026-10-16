use super::fixtures::*;
use crate::Compositor;
use sketch_core::TransformPreview;
use std::sync::Arc;
#[test]
fn cached_reference_and_analytic_alpha_preserve_order() {
    let (scene, request) = scene(vec![
        rectangle(1, 0, 4.0, [1.0, 0.0, 0.0, 0.5]),
        rectangle(2, 1, 8.0, [0.0, 0.0, 1.0, 0.5]),
    ]);
    let mut cache = Compositor::default();
    let (cold, stats) = render(&mut cache, &scene, &request, true);
    let (warm, reused) = render(&mut cache, &scene, &request, true);
    let (reference, _) = render(&mut Compositor::default(), &scene, &request, false);
    equivalent(&cold, &reference, "alpha-cold");
    equivalent(&warm, &reference, "alpha-warm");
    assert_eq!(stats.background_hit, 0);
    assert_eq!(reused.background_hit, 1);
    assert_eq!(pixel(&cold, 40, 40), &[255, 255, 255, 255]);
    assert_eq!(pixel(&cold, 5, 10), &[255, 127, 127, 255]);
    assert_eq!(pixel(&cold, 10, 10), &[127, 63, 191, 255]);
}
#[test]
fn transform_preview_stays_below_later_strokes_and_omits_culled_original() {
    let original = rectangle(1, 0, 4.0, [1.0, 0.0, 0.0, 0.5]);
    let mut replacement = original.clone();
    replacement.transform.tx = 4.0;
    let (mut scene, request) = scene(vec![original, rectangle(2, 1, 8.0, [0.0, 0.0, 1.0, 0.5])]);
    scene.transform_preview = Some(TransformPreview {
        stroke_id: 1,
        order: 0,
        replacement: Some(replacement),
    });
    let (cached, _) = render(&mut Compositor::default(), &scene, &request, true);
    let (reference, _) = render(&mut Compositor::default(), &scene, &request, false);
    equivalent(&cached, &reference, "transform-order");
    assert_eq!(pixel(&cached, 10, 10), &[127, 63, 191, 255]);
    assert_eq!(pixel(&cached, 5, 10), &[255, 255, 255, 255]);
    scene.transform_preview.as_mut().unwrap().replacement = None;
    let (culled, _) = render(&mut Compositor::default(), &scene, &request, true);
    assert_eq!(pixel(&culled, 10, 10), &[127, 127, 255, 255]);
}
#[test]
fn local_origin_preserves_small_geometry_at_billion_unit_coordinates() {
    let (scene, mut request) = scene(vec![rectangle(1, 0, 1_000_000_000.0, [0.0, 0.0, 0.0, 1.0])]);
    request.offset_x = -1_000_000_000.0 + 4.0;
    let (pixels, _) = render(&mut Compositor::default(), &scene, &request, true);
    assert_eq!(pixel(&pixels, 5, 5), &[0, 0, 0, 255]);
    assert_eq!(pixel(&pixels, 25, 5), &[255, 255, 255, 255]);
}
#[test]
fn one_fill_for_overlapping_actual_prediction_contours_and_atomic_commit() {
    let mut ink = rectangle(1, 0, 4.0, [1.0, 0.0, 0.0, 0.5]);
    let second = rectangle(1, 0, 8.0, [1.0, 0.0, 0.0, 0.5]);
    let commands = ink
        .geometry
        .commands
        .iter()
        .chain(second.geometry.commands.iter())
        .copied()
        .collect::<Vec<_>>();
    Arc::get_mut(&mut ink.geometry).unwrap().commands = commands.into();
    let (mut scene, mut request) = scene(vec![]);
    scene.live = Some(ink.clone());
    let mut compositor = Compositor::default();
    let (live, live_stats) = render(&mut compositor, &scene, &request, true);
    assert_eq!(pixel(&live, 10, 10), &[255, 127, 127, 255]);
    assert_eq!(live_stats.path_bytes, 0, "draft paths must not be retained");
    scene.live = None;
    scene.committed = vec![ink].into();
    scene.key.revision.0 += 1;
    request.revision += 1;
    let (committed, _) = render(&mut compositor, &scene, &request, true);
    equivalent(&live, &committed, "atomic-commit");
}
#[test]
fn negative_coordinate_alpha_matches_at_zoom_and_backing_density_matrix() {
    let (scene, base) = scene(vec![
        rectangle(1, 0, -16.0, [1.0, 0.0, 0.0, 0.5]),
        rectangle(2, 1, -12.0, [0.0, 0.0, 1.0, 0.5]),
    ]);
    for density in [1, 2, 3] {
        for zoom in [0.1, 0.75, 1.0, 16.0] {
            let mut request = base;
            request.pixel_width = 64 * density;
            request.pixel_height = 64 * density;
            request.zoom = zoom;
            request.offset_x = 32.0 + 10.0 * zoom;
            request.offset_y = 32.0 - 10.0 * zoom;
            let (cached, _) = render(&mut Compositor::default(), &scene, &request, true);
            let (reference, _) = render(&mut Compositor::default(), &scene, &request, false);
            equivalent(&cached, &reference, "negative-zoom-density");
            let center = ((32 * density * request.pixel_width + 32 * density) * 4) as usize;
            assert_eq!(&cached[center..center + 4], &[127, 63, 191, 255]);
        }
    }
}
