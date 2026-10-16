use crate::*;
impl WireApp {
    pub(crate) fn create(
        &mut self,
        config: SessionConfig,
        policy: SaveDestinationPolicy,
        fresh: u64,
        reply: &mut Reply,
    ) -> Result<(), WireError> {
        if self.sessions.len() >= self.limits.sessions {
            return Err(WireError::capacity());
        }
        let handle = self.app.create_session_with_save_policy(config, policy)?;
        self.sessions.push(crate::owner::Session {
            id: fresh,
            handle,
            effect: None,
        });
        reply.projection(fresh, self.app.projection(handle)?, 0);
        Ok(())
    }
    pub(crate) fn document_bounds(&self, id: u64, reply: &mut Reply) -> Result<(), WireError> {
        let handle = self.session(id)?;
        let bounds = self.app.document_bounds(handle)?;
        let state = self.app.projection(handle)?.editor;
        reply.u64(state.session.get());
        reply.u64(state.revision.0);
        reply.u32(u32::from(bounds.is_some()));
        reply.u32(0);
        let values = bounds.map_or([0.0; 4], |b| [b.x, b.y, b.width, b.height]);
        for value in values {
            reply.f64(value);
        }
        Ok(())
    }
}
