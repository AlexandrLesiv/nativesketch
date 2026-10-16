use crate::*;
pub(crate) fn ack(value: Acknowledgement) -> u32 {
    match value {
        Acknowledgement::SavedCurrent => 1,
        Acknowledgement::SavedOlder => 2,
        Acknowledgement::CopyWritten => 3,
        Acknowledgement::CheckpointWritten => 4,
        Acknowledgement::StaleSession => 5,
        Acknowledgement::StaleBinding => 6,
        Acknowledgement::Superseded => 7,
        Acknowledgement::Cancelled => 8,
        Acknowledgement::Failed => 9,
        Acknowledgement::CopyDispatched => 10,
    }
}
pub(crate) fn durability(value: Durability) -> u32 {
    match value {
        Durability::FileAndDirectorySynced => 1,
        Durability::DirectorySyncUnconfirmed => 2,
        Durability::ProviderConfirmed => 3,
    }
}
pub(crate) fn phase(value: FilePhase) -> u32 {
    match value {
        FilePhase::Idle => 0,
        FilePhase::Destination => 1,
        FilePhase::Source => 2,
        FilePhase::Unsaved => 3,
        FilePhase::Writes => 4,
        FilePhase::Replacement => 5,
        FilePhase::CloseReady => 6,
    }
}
pub(crate) fn invalidation(value: RenderInvalidation) -> u32 {
    match value {
        RenderInvalidation::None => 0,
        RenderInvalidation::Overlay => 1,
        RenderInvalidation::Full => 2,
    }
}
pub(crate) fn disabled(value: DisabledReason) -> u32 {
    match value {
        DisabledReason::ReadOnly => 1,
        DisabledReason::InteractionActive => 2,
        DisabledReason::WrongTool => 3,
        DisabledReason::NothingSelected => 4,
        DisabledReason::HistoryEmpty => 5,
        DisabledReason::NoActiveStroke => 6,
        DisabledReason::NoActiveTransform => 7,
    }
}
pub(crate) fn work(value: WorkKind) -> u32 {
    match value {
        WorkKind::Write => 1,
        WorkKind::Read => 2,
        WorkKind::PrepareNew => 3,
        WorkKind::Retire => 4,
    }
}
pub(crate) fn save_result(value: SaveResult) -> (u32, u32, u32) {
    match value {
        SaveResult::Written(v) => (1, durability(v), 0),
        SaveResult::Uncertain(v) => (2, 0, crate::tags::write_failure(v)),
        SaveResult::Failed(v) => (3, 0, crate::tags::write_failure(v)),
        SaveResult::CancelledBeforeStart => (4, 0, 0),
        SaveResult::WrittenAfterCancellation(v) => (5, durability(v), 0),
        SaveResult::FailedAfterCancellation(v) => (6, 0, crate::tags::write_failure(v)),
        SaveResult::UncertainAfterCancellation(v) => (7, 0, crate::tags::write_failure(v)),
        SaveResult::DispatchedUnconfirmed => (8, 0, 0),
        SaveResult::DispatchedAfterCancellation => (9, 0, 0),
    }
}
