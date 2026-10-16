use super::EditorSession;
use crate::{RenderScene, RenderViewport, SceneCache, SceneError, application::SceneKey};

impl EditorSession {
    /// Prepare outside input callbacks. Stable committed frames share an immutable root;
    /// callers separately bound snapshot lifetimes and own disposable cache state.
    pub fn render_scene(
        &self,
        cache: &mut SceneCache,
        viewport: RenderViewport,
    ) -> Result<RenderScene, SceneError> {
        cache.build(
            &self.editor.document,
            &self.editor.interaction,
            SceneKey {
                session: self.session,
                revision: self.view_state().revision,
                generation: self.generation,
            },
            viewport,
        )
    }
}
