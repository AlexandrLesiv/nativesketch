use super::*;
#[test]
fn diagnostics_are_typed_correlated_bounded_and_never_emit_input_samples() {
    let limits = AppLimits {
        operations: 2,
        ..AppLimits::default()
    };
    let (mut app, handle) = app(limits, true);
    stroke(&mut app, handle);
    assert!(app.take_diagnostic().is_none());
    for id in 1..=3 {
        let operation = save(&mut app, handle, resource(id), true);
        let encoded = write(&mut app);
        complete(
            &mut app,
            encoded
                .finish(WriteOutcome::Failed(StorageFailure::Write))
                .unwrap(),
        );
        app.apply_ready(handle, operation).unwrap();
    }
    assert_eq!(app.stats().diagnostic_drops, 1);
    let records = [
        app.take_diagnostic().unwrap(),
        app.take_diagnostic().unwrap(),
    ];
    assert!(app.take_diagnostic().is_none());
    for record in records {
        assert_eq!(record.session, Some(handle));
        assert_eq!(record.request, RequestId(42));
        assert_eq!(record.provider, Some(ProviderId(1)));
        assert!(record.operation.is_some());
        assert_eq!(
            record.outcome,
            DiagnosticOutcome::Write(SaveResult::Failed(StorageFailure::Write))
        );
        let redacted = format!("{record:?}");
        assert!(!redacted.contains("timestamp"));
        assert!(!redacted.contains("pressure"));
        assert!(!redacted.contains('/'));
        assert!(!redacted.contains("bytes"));
    }
}

#[test]
fn denied_resource_transfer_reports_opaque_provider_and_original_request() {
    let (mut app, handle) = app(AppLimits::default(), true);
    let outcome = run(&mut app, handle, Intent::File(FileIntent::SaveAs));
    let Some(HostEffect::ChooseDestination { token, .. }) = outcome.effect else {
        panic!()
    };
    let mut download = resource(1);
    download.access.confirmed_write = false;
    assert!(
        app.supply_resource(handle, ctx(&app, handle), token, download)
            .is_err()
    );
    let record = app.take_diagnostic().unwrap();
    assert_eq!(record.request, RequestId(42));
    assert_eq!(record.provider, Some(download.provider));
    assert_eq!(
        record.outcome,
        DiagnosticOutcome::Rejected(AppError::CapabilityDenied)
    );
    assert!(app.take_diagnostic().is_none());
}
