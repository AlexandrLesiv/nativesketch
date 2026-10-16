use crate::{
    dispatch::Choice,
    operations::{Operation, Phase},
    work::Retired,
    *,
};
impl PortableApp {
    /// All required operation/resource capacity is reserved before core mutation.
    pub(crate) fn capture(
        &mut self,
        handle: SessionHandle,
        request: RequestId,
        outcome: ApplicationOutcome,
        resource: Option<DestinationHandle>,
        reserved: Option<OperationId>,
    ) -> Result<AppOutcome, AppError> {
        let index = self.slot(handle)?;
        if outcome.retired.is_some()
            || outcome.save.is_some_and(|save| {
                save.token.purpose() == SavePurpose::SaveAs
                    && matches!(
                        save.acknowledgement,
                        Acknowledgement::SavedCurrent | Acknowledgement::SavedOlder
                    )
            })
        {
            self.sessions[index].bound_resource =
                outcome.projection.save.destination().map(|destination| {
                    let handle = resource.expect("issued binding capability retained");
                    assert_eq!(
                        self.resource(handle)
                            .expect("retained binding lease")
                            .destination,
                        destination
                    );
                    handle
                });
        }
        let mut phase = None;
        let mut held_resource = resource;
        let effect = match outcome.effect {
            Some(ApplicationEffect::ChooseDestination { token, purpose }) => {
                self.sessions[index].choice = Some(Choice::Destination(token, purpose));
                Some(HostEffect::ChooseDestination { token, purpose })
            }
            Some(ApplicationEffect::ChooseSource(token)) => {
                self.sessions[index].choice = Some(Choice::Source(token));
                Some(HostEffect::ChooseSource(token))
            }
            Some(ApplicationEffect::ConfirmUnsaved { token, revision }) => {
                Some(HostEffect::ConfirmUnsaved { token, revision })
            }
            Some(ApplicationEffect::AwaitWrites(token)) => Some(HostEffect::AwaitWrites(token)),
            Some(ApplicationEffect::NavigationReady(token)) => {
                Some(HostEffect::NavigationReady(token))
            }
            Some(ApplicationEffect::CloseConfirmed) => {
                self.sessions[index].close_confirmed = true;
                Some(HostEffect::CloseConfirmed)
            }
            Some(ApplicationEffect::Cancelled) => Some(HostEffect::Cancelled),
            Some(ApplicationEffect::Failure(failure)) => Some(HostEffect::Failure(failure)),
            Some(ApplicationEffect::SaveQueued(token)) => {
                if held_resource.is_none() {
                    held_resource = Some(if token.purpose() == SavePurpose::Save {
                        self.bound_write_resource(index)?
                    } else {
                        self.write_resource(token.destination(), token.purpose())?
                    });
                }
                phase = Some(Phase::Write(token));
                Some(HostEffect::WorkQueued(
                    reserved.expect("file capacity reserved"),
                ))
            }
            Some(ApplicationEffect::PrepareReplacement(ticket)) => {
                phase = Some(match ticket.source() {
                    ReplacementSource::New => Phase::New(ticket),
                    ReplacementSource::Document(id) | ReplacementSource::ViewSource(id) => {
                        if held_resource.is_none() {
                            held_resource = Some(self.destination(id)?);
                        }
                        Phase::Read(ticket)
                    }
                    ReplacementSource::Recovery(_) => unreachable!("recovery is native-v1 owned"),
                });
                Some(HostEffect::WorkQueued(
                    reserved.expect("replacement capacity reserved"),
                ))
            }
            None => None,
        };
        let slot = &mut self.sessions[index];
        if slot
            .choice
            .is_some_and(|c| Some(c.token()) != outcome.projection.continuation)
        {
            slot.choice = None;
        }
        if outcome.changes.document {
            slot.pristine = false;
        }
        if let Some(editor) = outcome.retired {
            assert!(phase.is_none());
            held_resource = None;
            phase = Some(Phase::Retire(RetiredState {
                stamp: None,
                states: vec![Retired::Editor(editor)],
            }));
        }
        if let Some(phase) = phase {
            let save = match &phase {
                Phase::Write(token) => Some(*token),
                _ => None,
            };
            let continuation = match &phase {
                Phase::Read(ticket) | Phase::New(ticket) => Some(ticket.token()),
                _ => None,
            };
            self.operations.push(Operation {
                id: reserved.expect("work capacity reserved"),
                session: handle,
                request,
                resource: held_resource,
                phase,
                save,
                continuation,
            });
        }
        self.release_unused();
        Ok(AppOutcome {
            projection: outcome.projection,
            changes: outcome.changes,
            invalidation: outcome.invalidation,
            effect,
            save: outcome.save,
        })
    }
}
