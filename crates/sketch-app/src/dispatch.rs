use crate::*;
#[derive(Clone, Copy)]
pub(crate) enum Choice {
    Source(EffectToken),
    Destination(EffectToken, SavePurpose),
}
impl Choice {
    pub fn token(self) -> EffectToken {
        match self {
            Self::Source(t) | Self::Destination(t, _) => t,
        }
    }
}
impl PortableApp {
    pub fn dispatch(
        &mut self,
        handle: SessionHandle,
        context: CommandContext,
        intent: Intent<'_>,
    ) -> Result<AppOutcome, AppError> {
        let index = self.attached(handle)?;
        let (command, reserved) = match intent {
            Intent::Edit(command) => (ApplicationCommand::Edit(command), None),
            Intent::Action(action) => (ApplicationCommand::Action(action), None),
            Intent::Input(batch) => (ApplicationCommand::Input(batch), None),
            Intent::File(intent) => {
                self.file_resource(index, intent)?;
                (
                    ApplicationCommand::File(intent),
                    Some(self.reserve_operation()?),
                )
            }
        };
        let outcome = self.sessions[index]
            .app
            .dispatch(&mut self.coordinator, context, command)?;
        self.capture(handle, context.request, outcome, None, reserved)
    }
    pub fn reply(
        &mut self,
        handle: SessionHandle,
        context: CommandContext,
        token: EffectToken,
        decision: EffectDecision,
    ) -> Result<AppOutcome, AppError> {
        if matches!(
            decision,
            EffectDecision::Destination(_) | EffectDecision::Source(_)
        ) {
            return Err(AppError::Invalid);
        }
        let index = self.attached(handle)?;
        if decision == EffectDecision::Unsaved(UnsavedChoice::Save) {
            self.file_resource(index, FileIntent::Save)?;
        }
        let reserved = if decision == EffectDecision::Cancel {
            None
        } else {
            Some(self.reserve_operation()?)
        };
        let outcome = self.sessions[index].app.dispatch(
            &mut self.coordinator,
            context,
            ApplicationCommand::ResolveEffect(EffectReply { token, decision }),
        )?;
        self.capture(handle, context.request, outcome, None, reserved)
    }
    fn file_resource(&self, index: usize, intent: FileIntent) -> Result<(), AppError> {
        match intent {
            FileIntent::Save => {
                if self.sessions[index]
                    .app
                    .projection()
                    .save
                    .destination()
                    .is_some()
                {
                    self.bound_write_resource(index)?;
                }
            }
            FileIntent::Checkpoint(id) => {
                self.write_resource(id, SavePurpose::Checkpoint)?;
            }
            FileIntent::Navigate(NavigationIntent::Recover(_)) => {
                return Err(AppError::Unsupported);
            }
            _ => {}
        }
        Ok(())
    }
}
