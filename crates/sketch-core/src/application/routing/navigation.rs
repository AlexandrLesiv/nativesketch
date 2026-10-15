use super::*;
impl ApplicationSession {
    pub(super) fn advance(
        &mut self,
        queue: &SaveCoordinator,
        token: EffectToken,
        discard_authorized: bool,
    ) -> ApplicationEffect {
        let state = self.editor.view_state();
        if self.explicit_pending() || queue.explicit_outstanding(state.session) != 0 {
            self.pending = Some(Pending::Writes(token));
            return ApplicationEffect::AwaitWrites(token);
        }
        if state.document.dirty && !discard_authorized {
            self.pending = Some(Pending::Unsaved(token, state.revision));
            return ApplicationEffect::ConfirmUnsaved {
                token,
                revision: state.revision,
            };
        }
        match self.navigation.unwrap().intent {
            NavigationIntent::Close => {
                self.pending = Some(Pending::CloseReady(token, state.revision));
                ApplicationEffect::NavigationReady(token)
            }
            NavigationIntent::Open => {
                self.pending = Some(Pending::Source(token, state.revision));
                ApplicationEffect::ChooseSource(token)
            }
            NavigationIntent::New => self.prepare(token, ReplacementSource::New),
            NavigationIntent::Recover(id) => self.prepare(token, ReplacementSource::Recovery(id)),
        }
    }
    pub(super) fn complete_save(
        &mut self,
        queue: &SaveCoordinator,
        completion: SaveCompletion,
        request: RequestId,
    ) -> (Acknowledgement, Option<ApplicationEffect>) {
        let token = completion.token();
        let result = completion.result();
        self.saves.retain(|issued| *issued != token);
        let acknowledgement = self.editor.acknowledge_save(completion);
        if !matches!(self.pending, Some(Pending::Writes(_))) {
            return (acknowledgement, None);
        }
        if self.navigation.is_some_and(|n| n.save == Some(token))
            && !matches!(result, SaveResult::Written(_))
        {
            self.pending = None;
            self.navigation = None;
            let failure = match result {
                SaveResult::Failed(f)
                | SaveResult::Uncertain(f)
                | SaveResult::FailedAfterCancellation(f)
                | SaveResult::UncertainAfterCancellation(f) => {
                    ApplicationEffect::Failure(ApplicationFailure::Write(f))
                }
                _ => ApplicationEffect::Cancelled,
            };
            return (acknowledgement, Some(failure));
        }
        if self.explicit_pending()
            || queue.explicit_outstanding(self.editor.view_state().session) != 0
        {
            return (acknowledgement, None);
        }
        let effect = match self.issue(request) {
            Ok(next) => self.advance(queue, next, false),
            Err(failure) => {
                self.pending = None;
                self.navigation = None;
                ApplicationEffect::Failure(failure)
            }
        };
        (acknowledgement, Some(effect))
    }
}
