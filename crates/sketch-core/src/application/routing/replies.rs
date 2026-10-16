use super::*;
impl ApplicationSession {
    pub(super) fn reply(
        &mut self,
        queue: &mut SaveCoordinator,
        context: CommandContext,
        reply: EffectReply,
    ) -> Result<ApplicationEffect, ApplicationFailure> {
        let pending = self
            .pending
            .filter(|p| p.token() == reply.token)
            .ok_or(ApplicationFailure::InvalidEffect)?;
        if reply.decision == EffectDecision::Cancel {
            self.pending = None;
            self.navigation = None;
            return Ok(ApplicationEffect::Cancelled);
        }
        // Validate the variant and stale authorization BEFORE consuming any continuation.
        let valid = matches!(
            (pending, reply.decision),
            (Pending::Destination(..), EffectDecision::Destination(_))
                | (Pending::Source(..), EffectDecision::Source(_))
                | (Pending::Unsaved(..), EffectDecision::Unsaved(_))
                | (Pending::Writes(..), EffectDecision::Continue)
                | (Pending::CloseReady(..), EffectDecision::ConfirmClose)
        );
        if !valid {
            return Err(ApplicationFailure::InvalidEffect);
        }
        let state = self.editor.view_state();
        if let Pending::Source(_, revision) = pending {
            if state.revision != revision {
                return Err(ApplicationFailure::StaleDocument);
            }
        }
        let next = self.issue(context.request)?;
        // Replies are explicit file decisions; invalid/replayed replies never cancel ink.
        self.cancel_interaction(context)?;
        Ok(match (pending, reply.decision) {
            (Pending::Destination(_, purpose), EffectDecision::Destination(destination)) => {
                self.pending = None;
                let effect = self.admit(queue, context.request, purpose, destination);
                if self.navigation.is_some() {
                    self.wait_for_save(next, &effect);
                }
                effect
            }
            (Pending::Source(..), EffectDecision::Source(destination)) => {
                self.prepare(next, ReplacementSource::Document(destination))
            }
            (Pending::Unsaved(_, revision), EffectDecision::Unsaved(choice)) => match choice {
                UnsavedChoice::Cancel => {
                    self.pending = None;
                    self.navigation = None;
                    ApplicationEffect::Cancelled
                }
                UnsavedChoice::Save => self.save_for_navigation(queue, next),
                UnsavedChoice::Discard => self.advance(queue, next, state.revision == revision),
            },
            (Pending::Writes(_), EffectDecision::Continue) => self.advance(queue, next, false),
            (Pending::CloseReady(_, revision), EffectDecision::ConfirmClose) => {
                if state.revision != revision
                    || self.explicit_pending()
                    || queue.explicit_outstanding(state.session) != 0
                {
                    self.advance(queue, next, false)
                } else {
                    self.pending = None;
                    self.navigation = None;
                    ApplicationEffect::CloseConfirmed
                }
            }
            _ => unreachable!(),
        })
    }
}
