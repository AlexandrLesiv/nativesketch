use super::*;
#[test]
fn failure_variants_map_to_all_fixed_domain_codes() {
    for (name, value) in [
        ("InvalidValue", FailureKind::InvalidValue),
        ("InvalidTransition", FailureKind::InvalidTransition),
        ("ReadOnly", FailureKind::ReadOnly),
        ("StaleSession", FailureKind::StaleSession),
        ("StaleInteraction", FailureKind::StaleInteraction),
        ("ResourceLimit", FailureKind::ResourceLimit),
        ("CorruptDocument", FailureKind::CorruptDocument),
        ("ServiceFailure", FailureKind::ServiceFailure),
    ] {
        assert_eq!(
            Failure::new(1, tags::failure_kind(value))
                .bytes()
                .as_slice(),
            fixture(&format!("failure.1.{name}"))
        );
    }
    for (name, value) in [
        ("Busy", StorageError::Busy),
        ("CapabilityDenied", StorageError::CapabilityDenied),
        ("InvalidBinding", StorageError::InvalidBinding),
        ("BindingPending", StorageError::BindingPending),
        ("ResourceLimit", StorageError::ResourceLimit),
        ("UnknownToken", StorageError::UnknownToken),
        ("InvalidTransition", StorageError::InvalidTransition),
    ] {
        assert_eq!(
            Failure::new(2, tags::storage_error(value))
                .bytes()
                .as_slice(),
            fixture(&format!("failure.2.{name}"))
        );
    }
    for (name, value) in [
        ("Encode", StorageFailure::Encode),
        ("Access", StorageFailure::Access),
        ("Capacity", StorageFailure::Capacity),
        ("Write", StorageFailure::Write),
        ("Replace", StorageFailure::Replace),
        ("WorkerStopped", StorageFailure::WorkerStopped),
    ] {
        assert_eq!(
            Failure::new(3, tags::write_failure(value))
                .bytes()
                .as_slice(),
            fixture(&format!("failure.3.{name}"))
        );
    }
    for (name, value) in [
        ("InvalidEffect", ApplicationFailure::InvalidEffect),
        ("StaleDocument", ApplicationFailure::StaleDocument),
        ("Busy", ApplicationFailure::Busy),
        ("CapabilityDenied", ApplicationFailure::CapabilityDenied),
        ("NavigationRequired", ApplicationFailure::NavigationRequired),
        ("ResourceLimit", ApplicationFailure::ResourceLimit),
    ] {
        assert_eq!(
            Failure::from(value).bytes().as_slice(),
            fixture(&format!("failure.4.{name}"))
        );
    }
    for (name, value) in [
        ("InvalidViewport", sketch_core::SceneError::InvalidViewport),
        ("ResourceLimit", sketch_core::SceneError::ResourceLimit),
    ] {
        assert_eq!(
            WireError::from(AppError::Scene(value))
                .failure
                .bytes()
                .as_slice(),
            fixture(&format!("failure.5.{name}"))
        );
    }
}
