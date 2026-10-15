use super::{owner_types::*, work::*};
use sketch_core::application::storage::SavePurpose;
impl OperationKind {
    pub(super) fn boundary(&self, state: &OperationState) -> u32 {
        match self {
            Self::Resolve { .. } => 203,
            Self::Prepare(_) => 204,
            Self::Save(token) => match token.purpose() {
                SavePurpose::Save => 20,
                SavePurpose::SaveAs => 21,
                SavePurpose::ExportCopy => 22,
                SavePurpose::Checkpoint => 205,
            },
            Self::Register => 205,
            Self::RetryRecovery => 209,
            Self::List => 206,
            Self::Retirement => 211,
            Self::Maintenance => match state {
                OperationState::Queued(work) => match work.as_ref() {
                    Work::Discard(_) => 207,
                    Work::ResolveRecovery { .. } => 208,
                    Work::Retry => 209,
                    Work::Diagnostics(_) => 210,
                    _ => 0,
                },
                _ => 0,
            },
        }
    }
}
