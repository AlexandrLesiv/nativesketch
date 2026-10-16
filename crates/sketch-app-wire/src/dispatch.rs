use crate::*;
impl WireApp {
    /// Validate completely, reserve reply and identities, then mutate once on this owner.
    pub fn call(&mut self, bytes: &[u8]) -> Result<Reply, WireError> {
        let packet = decode(bytes)?;
        if matches!(packet.opcode, 40..=47) {
            return self.scene_call(packet);
        }
        let mut reply = Reply::reserve(&self.budget, packet.opcode, packet.request, 4096)?;
        let fresh = crate::identity::reserve(3)?;
        match packet.command {
            Request::Create(config) => self.create(
                config,
                SaveDestinationPolicy::ReuseBinding,
                fresh,
                &mut reply,
            )?,
            Request::CreateWithSavePolicy(config, policy) => {
                self.create(config, policy, fresh, &mut reply)?
            }
            Request::DocumentBounds(id) => self.document_bounds(id, &mut reply)?,
            Request::Detach(id) => {
                let handle = self.session(id)?;
                self.app.detach_session(handle)?;
            }
            Request::Projection(id) => {
                let handle = self.session(id)?;
                let p = self.app.projection(handle)?;
                let c = self.track(handle, p, fresh);
                reply.projection(id, p, c);
            }
            Request::Availability(id, action) => {
                match self.app.availability(self.session(id)?, action)? {
                    Availability::Enabled => {
                        reply.u32(1);
                        reply.u32(0);
                    }
                    Availability::Disabled(reason) => {
                        reply.u32(0);
                        reply.u32(write_tags::disabled(reason));
                    }
                }
            }
            Request::Stats => self.stats_reply(&mut reply),
            Request::SelectedBounds(id) => {
                let b = self.app.selected_bounds(self.session(id)?)?;
                for v in [b.x, b.y, b.width, b.height] {
                    reply.f64(v);
                }
            }
            Request::Diagnostic => self.diagnostic(&mut reply),
            Request::Release => {
                let release = self.app.take_release();
                reply.u32(u32::from(release.is_some()));
                reply.u32(0);
                if let Some(value) = release {
                    reply.u64(fresh);
                    reply.resource(Some(value.descriptor));
                }
            }
            Request::Cancel(id) => self.cancel(id)?,
            Request::Apply(session, id) => {
                let handle = self.session(session)?;
                let operation = self.operation(id)?;
                let outcome = self.app.apply_ready(handle, operation);
                if outcome.is_ok()
                    || matches!(
                        outcome,
                        Err(AppError::Core(ApplicationError {
                            failure: ApplicationFailure::InvalidEffect
                                | ApplicationFailure::StaleDocument,
                            ..
                        }))
                    )
                {
                    self.operations.retain(|op| op.handle != operation);
                }
                self.encode_outcome(&mut reply, handle, outcome?, fresh);
            }
            command => {
                let (handle, outcome) = self.intent(command, packet.request)?;
                self.encode_outcome(&mut reply, handle, outcome, fresh);
            }
        }
        Ok(reply.finish())
    }
    fn cancel(&mut self, id: u64) -> Result<(), WireError> {
        let handle = self.operation(id)?;
        self.app.cancel_operation(handle)?;
        self.operations
            .retain(|op| op.handle != handle || op.active || op.write);
        Ok(())
    }
    fn intent(
        &mut self,
        command: Request,
        request: u64,
    ) -> Result<(SessionHandle, AppOutcome), WireError> {
        let (handle, outcome) = match command {
            Request::Edit(raw, command) => {
                let (h, c) = self.context(raw, request)?;
                (h, self.app.dispatch(h, c, Intent::Edit(command))?)
            }
            Request::Action(raw, action) => {
                let (h, c) = self.context(raw, request)?;
                (h, self.app.dispatch(h, c, Intent::Action(action))?)
            }
            Request::Input(raw, pointer, samples) => {
                let (h, c) = self.context(raw, request)?;
                (
                    h,
                    self.app.dispatch(
                        h,
                        c,
                        Intent::Input(SampleBatch {
                            pointer,
                            samples: &samples,
                        }),
                    )?,
                )
            }
            Request::File(raw, intent) => {
                let (h, c) = self.context(raw, request)?;
                (h, self.app.dispatch(h, c, Intent::File(intent))?)
            }
            Request::Reply(raw, id, decision) => {
                let (h, c) = self.context(raw, request)?;
                let token = self.effect(h, id)?;
                (h, self.app.reply(h, c, token, decision)?)
            }
            Request::Resource(raw, id, resource) => {
                let (h, c) = self.context(raw, request)?;
                let token = self.effect(h, id)?;
                (h, self.app.supply_resource(h, c, token, resource)?)
            }
            Request::ViewSource(raw, generation, resource) => {
                let (h, c) = self.context(raw, request)?;
                (h, self.app.begin_view_source(h, c, generation, resource)?)
            }
            Request::Unsupported => return Err(WireError::plain(13)),
            _ => return Err(WireError::transport(3)),
        };
        Ok((handle, outcome))
    }
}
