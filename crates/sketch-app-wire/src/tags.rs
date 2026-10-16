use crate::*;
pub(crate) fn operation(value: Operation) -> u32 {
    match value {
        Operation::Construct => 1,
        Operation::SetTool => 2,
        Operation::SetBrush => 3,
        Operation::Select => 4,
        Operation::BeginStroke => 5,
        Operation::Append => 6,
        Operation::CommitStroke => 7,
        Operation::BeginTransform => 8,
        Operation::UpdateTransform => 9,
        Operation::CommitTransform => 10,
        Operation::Cancel => 11,
        Operation::Delete => 12,
        Operation::Undo => 13,
        Operation::Redo => 14,
        Operation::New => 15,
        Operation::Replace => 16,
        Operation::Query => 17,
    }
}
pub(crate) fn failure_kind(value: FailureKind) -> u32 {
    match value {
        FailureKind::InvalidValue => 1,
        FailureKind::InvalidTransition => 2,
        FailureKind::ReadOnly => 3,
        FailureKind::StaleSession => 4,
        FailureKind::StaleInteraction => 5,
        FailureKind::ResourceLimit => 6,
        FailureKind::CorruptDocument => 7,
        FailureKind::ServiceFailure => 8,
    }
}
pub(crate) fn storage_error(value: StorageError) -> u32 {
    match value {
        StorageError::Busy => 1,
        StorageError::CapabilityDenied => 2,
        StorageError::InvalidBinding => 3,
        StorageError::BindingPending => 4,
        StorageError::ResourceLimit => 5,
        StorageError::UnknownToken => 6,
        StorageError::InvalidTransition => 7,
    }
}
pub(crate) fn write_failure(value: StorageFailure) -> u32 {
    match value {
        StorageFailure::Encode => 1,
        StorageFailure::Access => 2,
        StorageFailure::Capacity => 3,
        StorageFailure::Write => 4,
        StorageFailure::Replace => 5,
        StorageFailure::WorkerStopped => 6,
    }
}
pub(crate) fn purpose(value: SavePurpose) -> u32 {
    match value {
        SavePurpose::Save => 1,
        SavePurpose::SaveAs => 2,
        SavePurpose::Checkpoint => 3,
        SavePurpose::ExportCopy => 4,
    }
}
