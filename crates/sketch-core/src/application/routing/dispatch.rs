use super::*;
impl ApplicationSession {
    /// One mutation owner; save capture is only in explicit file/idle calls, never input.
    pub fn dispatch(
        &mut self,
        queue: &mut SaveCoordinator,
        context: CommandContext,
        command: ApplicationCommand<'_>,
    ) -> Result<ApplicationOutcome, ApplicationError> {
        self.editor
            .validate_context(context)
            .map_err(|error| ApplicationError {
                context,
                failure: ApplicationFailure::Editor(error),
            })?;
        let before = self.editor.view_state();
        let result = self.route(queue, context, command, before);
        result.map_err(|failure| ApplicationError { context, failure })
    }
    fn route(
        &mut self,
        queue: &mut SaveCoordinator,
        context: CommandContext,
        command: ApplicationCommand<'_>,
        before: EditorViewState,
    ) -> Result<ApplicationOutcome, ApplicationFailure> {
        let mut outcome = self.outcome(before, RenderInvalidation::None, None);
        match command {
            ApplicationCommand::Edit(command) => {
                if matches!(
                    command,
                    EditorCommand::NewDocument | EditorCommand::ReplaceDocument(_)
                ) {
                    return Err(ApplicationFailure::NavigationRequired);
                }
                let result = self
                    .editor
                    .execute(context, command)
                    .map_err(ApplicationFailure::Editor)?;
                outcome = self.outcome(before, result.invalidation, None);
                outcome.changes = result.changes;
            }
            ApplicationCommand::Action(action) => {
                let result = self
                    .editor
                    .action(context, action)
                    .map_err(ApplicationFailure::Editor)?;
                outcome = self.outcome(before, result.invalidation, None);
                outcome.changes = result.changes;
            }
            ApplicationCommand::Input(batch) => {
                let result = self
                    .editor
                    .append(context, batch)
                    .map_err(ApplicationFailure::Editor)?;
                outcome = self.outcome(before, result.invalidation, None);
                outcome.changes = result.changes;
            }
            ApplicationCommand::File(intent) => {
                let effect = self.file(queue, context, intent)?;
                outcome = self.outcome(before, RenderInvalidation::Full, Some(effect));
            }
            ApplicationCommand::ResolveEffect(reply) => {
                let effect = self.reply(queue, context, reply)?;
                outcome = self.outcome(before, RenderInvalidation::Full, Some(effect));
            }
            ApplicationCommand::CompleteSave(completion) => {
                let value = completion
                    .as_ref()
                    .ok_or(ApplicationFailure::InvalidEffect)?;
                let token = value.token();
                let result = value.result();
                if !self.saves.contains(&token) {
                    return Err(ApplicationFailure::InvalidEffect);
                }
                let (acknowledgement, effect) =
                    self.complete_save(queue, completion.take().unwrap(), context.request);
                outcome = self.outcome(before, RenderInvalidation::None, effect);
                outcome.save = Some(AppliedSave {
                    token,
                    result,
                    acknowledgement,
                });
            }
            ApplicationCommand::Install(candidate) => {
                let retired = self.install(candidate)?;
                outcome = self.outcome(before, RenderInvalidation::Full, None);
                outcome.retired = Some(retired);
            }
            ApplicationCommand::ReplacementFailed(failure) => {
                let value = failure.as_ref().ok_or(ApplicationFailure::InvalidEffect)?;
                if !matches!(self.pending, Some(Pending::Replacement(t, _)) if t == value.token) {
                    return Err(ApplicationFailure::InvalidEffect);
                }
                let value = failure.take().unwrap();
                self.pending = None;
                self.navigation = None;
                outcome.effect = Some(ApplicationEffect::Failure(value.failure));
                outcome.projection = self.projection();
            }
        }
        Ok(outcome)
    }
}
