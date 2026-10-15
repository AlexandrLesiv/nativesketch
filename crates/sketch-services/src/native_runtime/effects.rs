use super::{NativeRuntime, owner_types::*, work::*};
use sketch_core::application::{routing::*, storage::*};
impl NativeRuntime {
    pub(super) fn enqueue(
        &mut self,
        owner: OperationOwner,
        kind: OperationKind,
        state: OperationState,
        retry: u32,
    ) -> Result<u64, RuntimeError> {
        let OperationOwner {
            window,
            session,
            request,
            grant,
        } = owner;
        let id = self.id()?;
        if grant != 0 {
            self.retain_grant(grant)?;
        }
        self.operations.push_back(PendingOperation {
            id,
            window,
            session,
            request,
            grant,
            kind,
            state,
            event: None,
            retry,
            checkpoint: None,
            cancelled: false,
            items: Vec::new(),
        });
        Ok(id)
    }
    pub(super) fn capture_effect(
        &mut self,
        window: u64,
        effect: Option<ApplicationEffect>,
        grant: u64,
        retry: u32,
    ) -> Result<(IntentEffect, u64), RuntimeError> {
        let mut operation = 0;
        let effect = match effect {
            None => IntentEffect::None,
            Some(ApplicationEffect::ChooseDestination { token, purpose }) => {
                IntentEffect::Destination(token, purpose)
            }
            Some(ApplicationEffect::ChooseSource(token)) => IntentEffect::Source(token),
            Some(ApplicationEffect::ConfirmUnsaved { token, .. }) => IntentEffect::Unsaved(token),
            Some(ApplicationEffect::AwaitWrites(token)) => IntentEffect::Writes(token),
            Some(ApplicationEffect::SaveQueued(token)) => {
                let retry = match token.purpose() {
                    SavePurpose::Save => 20,
                    SavePurpose::SaveAs => 21,
                    SavePurpose::ExportCopy => 22,
                    SavePurpose::Checkpoint => 0,
                };
                operation = self.enqueue(
                    OperationOwner {
                        window,
                        session: token.session().get(),
                        request: token.request().0,
                        grant,
                    },
                    OperationKind::Save(token),
                    OperationState::WaitingWrite(token),
                    retry,
                )?;
                IntentEffect::Save(token)
            }
            Some(ApplicationEffect::PrepareReplacement(ticket)) => {
                let token = ticket.token();
                let recovery = if matches!(ticket.source(), ReplacementSource::Recovery(_)) {
                    self.sessions[self.slot(window)?].requested_recovery.clone()
                } else {
                    None
                };
                operation = self.enqueue(
                    OperationOwner {
                        window,
                        session: token.session().get(),
                        request: token.request().0,
                        grant,
                    },
                    OperationKind::Prepare(token),
                    OperationState::Queued(Box::new(Work::Prepare { ticket, recovery })),
                    retry,
                )?;
                IntentEffect::Opening
            }
            Some(ApplicationEffect::NavigationReady(token)) => IntentEffect::CloseReady(token),
            Some(ApplicationEffect::CloseConfirmed) => IntentEffect::CloseConfirmed,
            Some(ApplicationEffect::Cancelled) => IntentEffect::Cancelled,
            Some(ApplicationEffect::Failure(failure)) => IntentEffect::Failure(failure),
        };
        Ok((effect, operation))
    }
    pub(super) fn retire(&mut self, retired: Retired) -> Result<(), RuntimeError> {
        self.enqueue(
            OperationOwner {
                window: 0,
                session: 0,
                request: 0,
                grant: 0,
            },
            OperationKind::Retirement,
            OperationState::Queued(Box::new(Work::Retire(Box::new(retired)))),
            0,
        )?;
        Ok(())
    }
}
