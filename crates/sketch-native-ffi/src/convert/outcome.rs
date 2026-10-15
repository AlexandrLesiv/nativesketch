use super::{failures, projection};
use crate::dto;
use sketch_core::application::{RenderInvalidation, storage::*};
use sketch_services::native_runtime::{ActionResult, IntentEffect};
pub fn purpose(value: SavePurpose) -> u32 {
    match value {
        SavePurpose::Save => 1,
        SavePurpose::SaveAs => 2,
        SavePurpose::Checkpoint => 3,
        SavePurpose::ExportCopy => 4,
    }
}
fn durability(value: Durability) -> u32 {
    match value {
        Durability::FileAndDirectorySynced => 1,
        Durability::DirectorySyncUnconfirmed => 2,
        Durability::ProviderConfirmed => 3,
    }
}
pub fn outcome(value: ActionResult, current: dto::Projection) -> dto::Outcome {
    let window = current.window;
    let changes = value.outcome.changes;
    let mut result = dto::Outcome {
        projection: current,
        operation_id: value.operation,
        retry_action: value.retry_action,
        change_flags: u32::from(changes.document)
            | u32::from(changes.session_replaced) << 1
            | u32::from(changes.settings) << 2
            | u32::from(changes.selection) << 3
            | u32::from(changes.interaction) << 4,
        invalidation: match value.outcome.invalidation {
            RenderInvalidation::None => 0,
            RenderInvalidation::Overlay => 1,
            RenderInvalidation::Full => 2,
        },
        ..Default::default()
    };
    let (kind, token) = match value.effect {
        IntentEffect::None => (0, None),
        IntentEffect::Destination(token, p) => {
            result.save_purpose = purpose(p);
            (1, Some(token))
        }
        IntentEffect::Source(token) => (2, Some(token)),
        IntentEffect::Unsaved(token) => (3, Some(token)),
        IntentEffect::Writes(token) => (4, Some(token)),
        IntentEffect::Save(token) => {
            result.captured_session = token.session().get();
            result.captured_revision = token.revision().0;
            result.save_purpose = purpose(token.purpose());
            (5, None)
        }
        IntentEffect::Preparing => (6, None),
        IntentEffect::Opening => (7, None),
        IntentEffect::CloseReady(token) => (8, Some(token)),
        IntentEffect::CloseConfirmed => (9, None),
        IntentEffect::Cancelled => (10, None),
        IntentEffect::Failure(value) => {
            result.failure = failures::application(value);
            (11, None)
        }
    };
    result.effect_kind = kind;
    if let Some(token) = token {
        result.effect = projection::token(window, token);
    }
    if let Some(save) = value.outcome.save {
        result.has_save_result = 1;
        result.written_destination = save.token.destination().0;
        result.written_revision = save.token.revision().0;
        result.captured_session = save.token.session().get();
        result.captured_revision = save.token.revision().0;
        result.save_purpose = purpose(save.token.purpose());
        let (saved, failure, durable) = match save.result {
            SaveResult::Written(d) => (1, None, durability(d)),
            SaveResult::Uncertain(f) => (2, Some(f), 0),
            SaveResult::Failed(f) => (3, Some(f), 0),
            SaveResult::CancelledBeforeStart => (4, None, 0),
            SaveResult::WrittenAfterCancellation(d) => (5, None, durability(d)),
            SaveResult::FailedAfterCancellation(f) => (6, Some(f), 0),
            SaveResult::UncertainAfterCancellation(f) => (7, Some(f), 0),
        };
        result.save_result = saved;
        result.durability = durable;
        if let Some(failure) = failure {
            result.failure = dto::Failure {
                domain: 3,
                code: failures::storage(failure),
                ..Default::default()
            };
        }
        result.acknowledgement = match save.acknowledgement {
            Acknowledgement::SavedCurrent => 1,
            Acknowledgement::SavedOlder => 2,
            Acknowledgement::CopyWritten => 3,
            Acknowledgement::CheckpointWritten => 4,
            Acknowledgement::StaleSession => 5,
            Acknowledgement::StaleBinding => 6,
            Acknowledgement::Superseded => 7,
            Acknowledgement::Cancelled => 8,
            Acknowledgement::Failed => 9,
        };
    }
    result.failure.operation = value.operation_kind;
    result.failure.session = result.projection.session;
    result.failure.request = if value.request != 0 {
        value.request
    } else {
        result.effect.request
    };
    result.failure.operation_id = result.operation_id;
    result
}
