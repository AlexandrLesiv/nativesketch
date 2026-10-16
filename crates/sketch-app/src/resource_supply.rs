use crate::{dispatch::Choice, *};
impl PortableApp {
    pub fn supply_resource(
        &mut self,
        handle: SessionHandle,
        context: CommandContext,
        token: EffectToken,
        descriptor: ResourceDescriptor,
    ) -> Result<AppOutcome, AppError> {
        let result = self.supply_resource_inner(handle, context, token, descriptor);
        self.resource_rejected(handle, context, descriptor, &result);
        result
    }
    pub fn begin_view_source(
        &mut self,
        handle: SessionHandle,
        context: CommandContext,
        generation: u64,
        descriptor: ResourceDescriptor,
    ) -> Result<AppOutcome, AppError> {
        let result = self.begin_view_source_inner(handle, context, generation, descriptor);
        self.resource_rejected(handle, context, descriptor, &result);
        result
    }

    /// Successful return transfers the access lease, even if core reports a file failure
    /// effect. Err leaves the host lease owned by the caller; no release is emitted.
    fn supply_resource_inner(
        &mut self,
        handle: SessionHandle,
        context: CommandContext,
        token: EffectToken,
        descriptor: ResourceDescriptor,
    ) -> Result<AppOutcome, AppError> {
        let index = self.attached(handle)?;
        let choice = self.sessions[index]
            .choice
            .filter(|c| c.token() == token)
            .ok_or(AppError::Stale)?;
        let operation = self.reserve_operation()?;
        match choice {
            Choice::Source(_) if !descriptor.access.read => return Err(AppError::CapabilityDenied),
            Choice::Destination(_, purpose) => Self::check_write(descriptor.access, purpose)?,
            _ => {}
        }
        let resource = self.prepare_resource(descriptor)?;
        let decision = match choice {
            Choice::Source(_) => EffectDecision::Source(resource.destination),
            Choice::Destination(..) => EffectDecision::Destination(resource.destination),
        };
        let outcome = self.sessions[index].app.dispatch(
            &mut self.coordinator,
            context,
            ApplicationCommand::ResolveEffect(EffectReply { token, decision }),
        )?;
        let lease = resource.handle;
        self.resources.push(resource);
        self.capture(
            handle,
            context.request,
            outcome,
            Some(lease),
            Some(operation),
        )
    }
    fn begin_view_source_inner(
        &mut self,
        handle: SessionHandle,
        context: CommandContext,
        generation: u64,
        descriptor: ResourceDescriptor,
    ) -> Result<AppOutcome, AppError> {
        let index = self.attached(handle)?;
        if !descriptor.access.read {
            return Err(AppError::CapabilityDenied);
        }
        let operation = self.reserve_operation()?;
        let resource = self.prepare_resource(descriptor)?;
        let outcome = self.sessions[index].app.dispatch(
            &mut self.coordinator,
            context,
            ApplicationCommand::ViewSource {
                source: resource.destination,
                generation,
            },
        )?;
        let lease = resource.handle;
        self.resources.push(resource);
        self.capture(
            handle,
            context.request,
            outcome,
            Some(lease),
            Some(operation),
        )
    }
}
