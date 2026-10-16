use crate::dto;
use sketch_core::application::{FailureKind, routing::ApplicationFailure, storage::*};
use sketch_services::native_runtime::{RuntimeError, work::NativeFailure};
pub fn status(error: RuntimeError) -> u32 {
    match error {
        RuntimeError::Invalid => 1,
        RuntimeError::Stale => 7,
        RuntimeError::Busy => 9,
        RuntimeError::Capacity => 3,
        RuntimeError::Owner => 11,
        RuntimeError::Stopped => 12,
        RuntimeError::Denied => 8,
        RuntimeError::Application(failure) => match failure {
            ApplicationFailure::Editor(error) => match error.kind {
                FailureKind::InvalidValue => 1,
                FailureKind::InvalidTransition => 2,
                FailureKind::ReadOnly => 8,
                FailureKind::StaleSession | FailureKind::StaleInteraction => 7,
                FailureKind::ResourceLimit => 3,
                FailureKind::CorruptDocument => 5,
                FailureKind::ServiceFailure => 4,
            },
            ApplicationFailure::CapabilityDenied => 8,
            ApplicationFailure::Busy => 9,
            ApplicationFailure::ResourceLimit => 3,
            ApplicationFailure::StaleDocument | ApplicationFailure::InvalidEffect => 7,
            ApplicationFailure::Storage(_) | ApplicationFailure::Write(_) => 4,
            ApplicationFailure::NavigationRequired => 2,
        },
    }
}
pub fn storage(value: StorageFailure) -> u32 {
    match value {
        StorageFailure::Encode => 1,
        StorageFailure::Access => 2,
        StorageFailure::Capacity => 3,
        StorageFailure::Write => 4,
        StorageFailure::Replace => 5,
        StorageFailure::WorkerStopped => 6,
    }
}
pub fn application(value: ApplicationFailure) -> dto::Failure {
    let (domain, code) = match value {
        ApplicationFailure::Editor(value) => (
            1,
            match value.kind {
                FailureKind::InvalidValue => 1,
                FailureKind::InvalidTransition => 2,
                FailureKind::ReadOnly => 3,
                FailureKind::StaleSession => 4,
                FailureKind::StaleInteraction => 5,
                FailureKind::ResourceLimit => 6,
                FailureKind::CorruptDocument => 7,
                FailureKind::ServiceFailure => 8,
            },
        ),
        ApplicationFailure::Storage(value) => (
            2,
            match value {
                StorageError::Busy => 1,
                StorageError::CapabilityDenied => 2,
                StorageError::InvalidBinding => 3,
                StorageError::BindingPending => 4,
                StorageError::ResourceLimit => 5,
                StorageError::UnknownToken => 6,
                StorageError::InvalidTransition => 7,
            },
        ),
        ApplicationFailure::Write(value) => (3, storage(value)),
        ApplicationFailure::InvalidEffect => (4, 1),
        ApplicationFailure::StaleDocument => (4, 2),
        ApplicationFailure::Busy => (4, 3),
        ApplicationFailure::CapabilityDenied => (4, 4),
        ApplicationFailure::NavigationRequired => (4, 5),
        ApplicationFailure::ResourceLimit => (4, 6),
    };
    dto::Failure {
        domain,
        code,
        ..Default::default()
    }
}
pub fn native(value: NativeFailure) -> dto::Failure {
    let (domain, code) = match value {
        NativeFailure::Access => (5, 1),
        NativeFailure::Capacity => (5, 2),
        NativeFailure::Stopped => (5, 3),
        NativeFailure::Panic => (5, 4),
        NativeFailure::Unsupported => (5, 5),
        NativeFailure::Corrupt => (5, 6),
        NativeFailure::Stale => (5, 7),
        NativeFailure::Sink => (5, 8),
        NativeFailure::Recovery(error) => (
            6,
            match error {
                sketch_services::recovery::RecoveryError::Invalid => 3,
                sketch_services::recovery::RecoveryError::UnsafeEntry => 1,
                sketch_services::recovery::RecoveryError::Capacity => 2,
                sketch_services::recovery::RecoveryError::Conflict => 4,
                sketch_services::recovery::RecoveryError::Io => 1,
                sketch_services::recovery::RecoveryError::Missing => 7,
            },
        ),
    };
    dto::Failure {
        domain,
        code,
        ..Default::default()
    }
}
