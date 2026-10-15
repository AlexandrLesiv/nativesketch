use super::*;
impl EditorSession {
    pub(super) fn check_context(
        &self,
        context: CommandContext,
        transaction_bound: bool,
    ) -> Result<(), FailureKind> {
        if context.session != self.session {
            return Err(FailureKind::StaleSession);
        }
        if (transaction_bound || context.interaction.is_some())
            && context.interaction != self.transaction
        {
            return Err(FailureKind::StaleInteraction);
        }
        Ok(())
    }
    pub(super) fn next_generation(&self) -> Result<SceneGeneration, FailureKind> {
        self.generation
            .0
            .checked_add(1)
            .map(SceneGeneration)
            .ok_or(FailureKind::ResourceLimit)
    }
    pub(super) fn require_idle(&self) -> Result<(), FailureKind> {
        if self.transaction.is_none() {
            Ok(())
        } else {
            Err(FailureKind::InvalidTransition)
        }
    }
    pub(super) fn require_tool(&self, tool: ToolId) -> Result<(), FailureKind> {
        if self.tool == tool {
            Ok(())
        } else {
            Err(FailureKind::InvalidTransition)
        }
    }
}
