use super::*;
#[test]
fn flat_diagnostic_failures_keep_status_and_full_fixed_layout() {
    let a = app();
    for (name, error) in [
        ("capacity", AppError::Capacity),
        ("busy", AppError::Busy),
        ("unsupported", AppError::Unsupported),
    ] {
        let event = AppDiagnostic {
            app: 9,
            operation: None,
            session: None,
            request: RequestId(17),
            provider: None,
            outcome: DiagnosticOutcome::Rejected(error),
        };
        let mut reply = Reply::reserve(&a.budget, 7, 0xfedcba9876543210, 4096).unwrap();
        reply.u32(1);
        reply.u32(0);
        reply.diagnostic_record(event, 0);
        assert_eq!(
            reply.finish().bytes(),
            fixture(&format!("response.diagnostic.{name}"))
        );
    }
}
#[test]
fn diagnostic_kinds_and_empty_unused_fields_are_discriminated() {
    let a = app();
    for (name, outcome) in [
        ("Completed", DiagnosticOutcome::WorkCompleted),
        (
            "Write",
            DiagnosticOutcome::Write(SaveResult::Written(Durability::ProviderConfirmed)),
        ),
        (
            "ReplacementFailed",
            DiagnosticOutcome::ReplacementFailed(ApplicationFailure::Busy),
        ),
        ("Cancelled", DiagnosticOutcome::Cancelled),
        ("Rejected", DiagnosticOutcome::Rejected(AppError::Capacity)),
    ] {
        let event = AppDiagnostic {
            app: 9,
            operation: None,
            session: None,
            request: RequestId(17),
            provider: None,
            outcome,
        };
        let mut reply = Reply::reserve(&a.budget, 7, 1, 4096).unwrap();
        reply.diagnostic_record(event, 0);
        let reply = reply.finish();
        assert_eq!(
            &reply.bytes()[108..112],
            fixture(&format!("tag.diagnostic.{name}"))
        );
        if name != "Write" && name != "Rejected" {
            assert_eq!(u32_at(reply.bytes(), 112), 0);
        }
        if name != "Write" {
            assert_eq!(u32_at(reply.bytes(), 116), 0);
            assert_eq!(u32_at(reply.bytes(), 120), 0);
        }
    }
}
