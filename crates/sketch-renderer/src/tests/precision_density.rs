use super::{fixtures::*, precision::editor_stroke};
use crate::Compositor;
use sketch_core::SceneCache;
#[test]
fn huge_endpoint_uses_actual_fractional_asymmetric_backing_ratios() {
    let editor = editor_stroke(0.0, 999_999_010.0);
    let (_, mut request) = scene(vec![]);
    let state = editor.view_state();
    request.session = state.session.get();
    request.revision = state.revision.0;
    request.scene_generation = state.scene_generation.get();
    request.offset_x = -999_999_000.0;
    request.pixel_width = 85;
    request.pixel_height = 111;
    let scene = editor
        .render_scene(&mut SceneCache::default(), request.viewport())
        .unwrap();
    let (cached, _) = render(&mut Compositor::default(), &scene, &request, true);
    let (reference, _) = render(&mut Compositor::default(), &scene, &request, false);
    equivalent(&cached, &reference, "fractional-backing-endpoint");
    let pixel = |x: usize, y: usize| &cached[(y * 85 + x) * 4..(y * 85 + x + 1) * 4];
    assert_eq!(pixel(12, 35), &[0, 0, 0, 255]);
    assert_eq!(pixel(17, 35), &[255, 255, 255, 255]);
}
