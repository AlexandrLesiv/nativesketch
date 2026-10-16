use super::fixtures::*;
use crate::{Compositor, RenderError};
#[test]
fn invalid_and_stale_requests_do_not_change_cache_state() {
    let (scene, request) = scene(vec![rectangle(1, 0, 4.0, [0.0, 0.0, 0.0, 1.0])]);
    let mut compositor = Compositor::default();
    render(&mut compositor, &scene, &request, true);
    let mut bad = request;
    bad.zoom = f64::NAN;
    assert_eq!(compositor.validate(&bad), Err(RenderError::InvalidRequest));
    bad = request;
    bad.surface_generation = 0;
    assert_eq!(compositor.validate(&bad), Err(RenderError::InvalidRequest));
    bad = request;
    bad.pixel_width += 1;
    assert_eq!(compositor.validate(&bad), Err(RenderError::InvalidRequest));
    let (_, stats) = render(&mut compositor, &scene, &request, true);
    assert_eq!(stats.background_hit, 1);
    let mut newer = request;
    newer.surface_generation = 2;
    render(&mut compositor, &scene, &newer, true);
    assert_eq!(compositor.validate(&request), Err(RenderError::Stale));
}
#[test]
fn reset_and_budget_bypass_preserve_pixels_without_retaining_oversized_resources() {
    let (scene, request) = scene(vec![rectangle(1, 0, 4.0, [0.0, 0.0, 0.0, 1.0])]);
    let mut cache = Compositor::default();
    let (before, stats) = render(&mut cache, &scene, &request, true);
    cache.reset().unwrap();
    let (after, reset) = render(&mut cache, &scene, &request, true);
    equivalent(&before, &after, "reset");
    assert_eq!(reset.context_generation, stats.context_generation + 1);
    assert_eq!(reset.background_hit, 0);
    let (bypass, limited) = render(&mut Compositor::new(0, 0), &scene, &request, true);
    equivalent(&before, &bypass, "budget-bypass");
    assert_eq!(limited.path_bytes, 0);
    assert_eq!(limited.background_bytes, 0);
}
#[test]
fn new_session_with_reused_id_and_viewport_density_replaces_cached_content() {
    let (first, request) = scene(vec![rectangle(1, 0, 4.0, [1.0, 0.0, 0.0, 1.0])]);
    let mut compositor = Compositor::default();
    render(&mut compositor, &first, &request, true);
    let (second, mut new_request) = scene(vec![rectangle(1, 0, 30.0, [0.0, 0.0, 1.0, 1.0])]);
    let (replaced, _) = render(&mut compositor, &second, &new_request, true);
    assert_eq!(pixel(&replaced, 5, 10), &[255, 255, 255, 255]);
    assert_eq!(pixel(&replaced, 31, 10), &[0, 0, 255, 255]);
    new_request.surface_generation += 1;
    new_request.pixel_width = 128;
    new_request.pixel_height = 128;
    new_request.zoom = 0.75;
    new_request.offset_x = -4.0;
    let (scaled, _) = render(&mut compositor, &second, &new_request, true);
    let (reference, _) = render(&mut Compositor::default(), &second, &new_request, false);
    equivalent(&scaled, &reference, "density-zoom");
}
