use sketch_core::SceneCache;
use sketch_renderer::{MetalRenderer, RenderError};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    thread::{self, ThreadId},
};
pub struct Renderer {
    pub owner: ThreadId,
    pub poisoned: bool,
    pub scene: SceneCache,
    pub selection: crate::renderer_selection::SelectionCache,
    pub backend: MetalRenderer,
}
pub fn status(error: RenderError) -> u32 {
    match error {
        RenderError::InvalidRequest => 1,
        RenderError::Stale => 7,
        RenderError::ResourceLimit => 3,
        RenderError::Backend => 2,
    }
}
pub unsafe fn run(
    pointer: *mut Renderer,
    body: impl FnOnce(&mut Renderer) -> Result<(), u32>,
) -> u32 {
    if pointer.is_null() {
        return 1;
    }
    if unsafe { (*pointer).owner } != thread::current().id() {
        return 11;
    }
    let renderer = unsafe { &mut *pointer };
    if renderer.poisoned {
        return 10;
    }
    match catch_unwind(AssertUnwindSafe(|| body(renderer))) {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => error,
        Err(_) => {
            renderer.poisoned = true;
            6
        }
    }
}
