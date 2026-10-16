use super::ApplicationSession;
use crate::{RenderScene, RenderViewport, SceneCache, SceneError};

impl ApplicationSession {
    /// Immutable rendering projection; cache ownership is external to application state.
    pub fn render_scene(
        &self,
        cache: &mut SceneCache,
        viewport: RenderViewport,
    ) -> Result<RenderScene, SceneError> {
        self.editor.render_scene(cache, viewport)
    }
}
