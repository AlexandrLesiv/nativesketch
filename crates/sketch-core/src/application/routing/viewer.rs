use super::*;
impl ApplicationSession {
    /// Issued ticket ownership is identical to ordinary loads, but this is not edit or
    /// export authority. The provider must also apply its own document/version policy.
    pub(super) fn view_source(
        &mut self,
        context: CommandContext,
        source: DestinationId,
        generation: u64,
    ) -> Result<ApplicationEffect, ApplicationFailure> {
        let state = self.editor.view_state();
        if state.capabilities.edit_document || state.capabilities.export_document {
            return Err(ApplicationFailure::CapabilityDenied);
        }
        if generation <= self.view_source_generation {
            return Err(ApplicationFailure::StaleDocument);
        }
        if source.0 == 0 {
            return Err(ApplicationFailure::InvalidEffect);
        }
        // Only another viewer replacement may be superseded. A close continuation
        // belongs to its existing lifecycle and must be explicitly resolved first.
        if self.pending.is_some() && !matches!(self.pending, Some(Pending::Replacement(..)))
            || self.navigation.is_some()
            || self.explicit_pending()
            || state.interaction != InteractionState::Idle
        {
            return Err(ApplicationFailure::Busy);
        }
        let token = self.issue(context.request)?;
        self.view_source_generation = generation;
        Ok(self.prepare(token, ReplacementSource::ViewSource(source)))
    }
}
