use crate::*;
impl WireApp {
    pub(crate) fn scene_call(&mut self, packet: Packet) -> Result<Reply, WireError> {
        let maximum = match packet.command {
            Request::Descriptors(_, _, limit) => {
                80 + usize::try_from(limit.min(1365)).expect("bounded page") * 48
            }
            Request::GeometryCommands(_, _, _, limit) | Request::PreparedCommands(_, _, limit) => {
                80 + usize::try_from(limit.min(744)).expect("bounded page") * 88
            }
            _ => 4096,
        };
        let mut reply = Reply::reserve(&self.budget, packet.opcode, packet.request, maximum)?;
        match packet.command {
            Request::Scene(id, previous, viewport) => {
                let handle = self.session(id)?;
                let p = self.app.projection(handle)?.editor;
                if previous != 0 {
                    let prior = self.scenes.lease(previous)?;
                    if prior.session != handle {
                        return Err(WireError::plain(7));
                    }
                    if prior.scene.key.session == p.session
                        && prior.scene.key.revision == p.revision
                        && prior.scene.key.generation == p.scene_generation
                        && prior.viewport == viewport
                    {
                        self.scenes.info(&mut reply, previous, true)?;
                        return Ok(reply.finish());
                    }
                }
                if self.scenes.leases.len() >= self.scenes.limits.scenes {
                    return Err(WireError::capacity());
                }
                let fresh = crate::identity::reserve(sketch_core::MAX_STROKES as u64 + 4)?;
                let scene = self.app.scene(handle, &mut self.scenes.cache, viewport)?;
                let id = self.scenes.publish(handle, viewport, scene, fresh)?;
                self.scenes.info(&mut reply, id, false)?;
            }
            Request::Descriptors(id, first, limit) => {
                self.scenes.descriptors(&mut reply, id, first, limit)?
            }
            Request::GeometryInfo(scene, id) => self.scenes.geometry_info(&mut reply, scene, id)?,
            Request::GeometryCommands(scene, id, first, limit) => {
                self.scenes.commands(&mut reply, scene, id, first, limit)?
            }
            Request::ReleaseScene(id) => self.scenes.release(id)?,
            Request::PrepareViewport(scene, selector, index, width, height) => {
                self.scenes
                    .prepare_viewport(&mut reply, scene, selector, index, width, height)?;
            }
            Request::PreparedCommands(id, first, limit) => {
                self.scenes
                    .prepared_commands(&mut reply, id, first, limit)?;
            }
            Request::ReleasePrepared(id) => self.scenes.release_prepared(id)?,
            _ => return Err(WireError::transport(3)),
        }
        Ok(reply.finish())
    }
}
