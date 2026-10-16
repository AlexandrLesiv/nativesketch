use super::*;
impl EditorSession {
    /// High-level intent: validate and reserve first, then cancel+apply atomically.
    /// Primitive execute(Undo) continues to reject active interactions (UNDO-002).
    pub fn action(
        &mut self,
        context: CommandContext,
        action: EditorAction,
    ) -> Result<CommandOutcome, EditorError> {
        let fail = |kind| EditorError {
            operation: action.operation(),
            kind,
            context: Some(context),
        };
        self.check_context(context, false).map_err(fail)?;
        let before = self.view_state();
        match before.action_availability(action) {
            Availability::Enabled => {}
            Availability::Disabled(DisabledReason::ReadOnly) => {
                return Err(fail(FailureKind::ReadOnly));
            }
            Availability::Disabled(_) => return Err(fail(FailureKind::InvalidTransition)),
        }
        let generation = self.next_generation().map_err(fail)?;
        let prepared = self.editor.prepare_action(action).map_err(fail)?;
        // No recoverable errors beyond this point; no intermediate projection escapes.
        if let Some(tool) = self.editor.apply_action(prepared) {
            self.tool = tool;
        }
        self.transaction = None;
        self.generation = generation;
        Ok(self.outcome(before, RenderInvalidation::Full, false))
    }
}
