use crate::{
    Runtime, Session, boundary,
    renderer_owner::{self, Renderer},
};
use sketch_core::{
    SceneCache, SceneError,
    application::{EditorQuery, QueryResult},
};
use sketch_renderer::{MetalRenderer, RenderRequest, RenderStats};
use std::{ffi::c_void, thread, time::Instant};
/// # Safety
/// Valid aligned output; device and queue must be valid Metal objects on this owner thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_renderer_v1_create(
    device: *const c_void,
    queue: *const c_void,
    output: *mut *mut Renderer,
) -> u32 {
    boundary::entry(|| {
        boundary::require_output(output)?;
        unsafe {
            output.write(std::ptr::null_mut());
        }
        let backend =
            unsafe { MetalRenderer::new(device, queue) }.map_err(renderer_owner::status)?;
        let handle = Renderer {
            owner: thread::current().id(),
            poisoned: false,
            scene: SceneCache::default(),
            backend,
        };
        unsafe {
            output.write(Box::into_raw(Box::new(handle)));
        }
        Ok(())
    })
}
/// # Safety
/// Live serialized handles; aligned request/output; texture is a valid drawable Metal texture.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_renderer_v1_draw(
    runtime: *mut Runtime,
    session: *mut Session,
    renderer: *mut Renderer,
    texture: *const c_void,
    request: *const RenderRequest,
    output: *mut RenderStats,
) -> u32 {
    unsafe {
        boundary::runtime(runtime, false, |value| {
            boundary::require_output(output)?;
            let request = boundary::read(request)?;
            request.validate().map_err(renderer_owner::status)?;
            if texture.is_null() {
                return Err(1);
            }
            let session = boundary::session(session, runtime)?;
            let slot = boundary::status(value.core.session(session.window))?;
            let state = slot.app.projection().editor;
            if request.session != state.session.get()
                || request.revision != state.revision.0
                || request.scene_generation != state.scene_generation.get()
            {
                return Err(7);
            }
            let result = renderer_owner::run(renderer, |renderer| {
                renderer
                    .backend
                    .validate_texture(texture, &request)
                    .map_err(renderer_owner::status)?;
                let started = Instant::now();
                let scene = slot
                    .app
                    .render_scene(&mut renderer.scene, request.viewport())
                    .map_err(|error| match error {
                        SceneError::InvalidViewport => 1u32,
                        SceneError::ResourceLimit => 3,
                    })?;
                let selected = if state.document.selected != 0 {
                    match slot
                        .app
                        .query(EditorQuery::SelectedBounds)
                        .map_err(|_| 2u32)?
                    {
                        QueryResult::SelectedBounds(bounds) => Some(bounds),
                        _ => return Err(2),
                    }
                } else {
                    None
                };
                let scene_ns = started.elapsed().as_nanos().min(u64::MAX as u128) as u64;
                let render_start = Instant::now();
                let mut stats = renderer
                    .backend
                    .draw(texture, &scene, &request, selected)
                    .map_err(renderer_owner::status)?;
                stats.scene_prepare_ns = scene_ns;
                stats.render_submit_ns =
                    render_start.elapsed().as_nanos().min(u64::MAX as u128) as u64;
                stats.geometry_bytes = renderer.scene.stats().retained_bytes as u64;
                output.write(stats);
                Ok(())
            });
            if result == 0 { Ok(()) } else { Err(result) }
        })
    }
}
/// # Safety
/// Renderer is a live uniquely owned handle on its creation thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_renderer_v1_reset(renderer: *mut Renderer) -> u32 {
    unsafe {
        renderer_owner::run(renderer, |renderer| {
            renderer.backend.reset().map_err(renderer_owner::status)?;
            renderer.scene.clear();
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned pointer to a live owned handle or null, on its creation thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_renderer_v1_free(pointer: *mut *mut Renderer) -> u32 {
    boundary::entry(|| {
        let handle = unsafe { boundary::read(pointer)? };
        if handle.is_null() {
            return Ok(());
        }
        if unsafe { (*handle).owner } != thread::current().id() {
            return Err(11);
        }
        unsafe {
            drop(Box::from_raw(handle));
            pointer.write(std::ptr::null_mut());
        }
        Ok(())
    })
}
