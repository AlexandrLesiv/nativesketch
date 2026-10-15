use crate::renderer_api::*;
use sketch_renderer::{RenderRequest, RenderStats};
#[test]
fn renderer_abi_layout_matches_c_without_creating_a_gpu_context() {
    assert_eq!(size_of::<RenderRequest>(), 152);
    assert_eq!(std::mem::offset_of!(RenderRequest, selection_rgba), 88);
    assert_eq!(size_of::<RenderStats>(), 72);
    assert_eq!(std::mem::offset_of!(RenderStats, background_hit), 64);
}
#[test]
fn invalid_handles_are_rejected_without_touching_metal() {
    unsafe {
        let mut handle = std::ptr::null_mut();
        assert_eq!(
            sketch_renderer_v1_create(std::ptr::null(), std::ptr::null(), &mut handle),
            1
        );
        assert!(handle.is_null());
        assert_eq!(sketch_renderer_v1_reset(handle), 1);
        assert_eq!(sketch_renderer_v1_free(&mut handle), 0);
        assert_eq!(sketch_renderer_v1_free(std::ptr::null_mut()), 1);
        let mut stats = RenderStats::default();
        assert_eq!(
            sketch_renderer_v1_draw(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                handle,
                std::ptr::null(),
                &RenderRequest::default(),
                &mut stats
            ),
            1
        );
    }
}
