use crate::*;
#[derive(Debug, Clone, Copy)]
pub enum HostEffect {
    ChooseDestination {
        token: EffectToken,
        purpose: SavePurpose,
    },
    ChooseSource(EffectToken),
    ConfirmUnsaved {
        token: EffectToken,
        revision: Revision,
    },
    AwaitWrites(EffectToken),
    WorkQueued(OperationId),
    NavigationReady(EffectToken),
    CloseConfirmed,
    Cancelled,
    Failure(ApplicationFailure),
}
#[derive(Debug)]
pub struct AppOutcome {
    pub projection: ApplicationProjection,
    pub changes: StateChanges,
    pub invalidation: RenderInvalidation,
    pub effect: Option<HostEffect>,
    pub save: Option<AppliedSave>,
}
