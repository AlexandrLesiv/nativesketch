use super::*;
#[test]
fn all_failure_domain_codes_have_exact_golden_bytes() {
    for (name, bytes) in fixtures()
        .into_iter()
        .filter(|(n, _)| n.starts_with("failure."))
    {
        let domain = name.split('.').nth(1).unwrap().parse().unwrap();
        let code = u32_at(&bytes, 4);
        let f = Failure::new(domain, code);
        assert_eq!(f.bytes().as_slice(), bytes);
    }
    for (name, bytes) in fixtures()
        .into_iter()
        .filter(|(n, _)| n.starts_with("tag.status."))
    {
        let status = u32_at(&bytes, 0);
        let packet = WireError::plain(status).packet(5, 99);
        assert_eq!(&packet[24..28], bytes);
        assert_eq!(u64_at(&packet, 16), 99, "{name}");
        assert_eq!(u32_at(&packet, 8), 0x80000005);
    }
}
#[test]
fn editor_failure_context_and_router_status_are_redacted() {
    let mut app = app();
    let id = session(&mut app);
    let ctx = app
        .context(
            RawContext {
                session: id,
                document: app
                    .app
                    .projection(app.session(id).unwrap())
                    .unwrap()
                    .editor
                    .session
                    .get(),
                interaction: 0,
            },
            42,
        )
        .unwrap()
        .1;
    let err = EditorError {
        operation: Operation::CommitStroke,
        kind: FailureKind::InvalidTransition,
        context: Some(ctx),
    };
    let wire = WireError::from(AppError::Editor(err));
    assert_eq!(wire.status, 2);
    assert_eq!(wire.failure.domain, 1);
    assert_eq!(wire.failure.code, 2);
    assert_eq!(wire.failure.operation, 7);
    assert_eq!(wire.failure.request, 42);
    let wire = WireError::from(AppError::Core(ApplicationError {
        context: ctx,
        failure: ApplicationFailure::StaleDocument,
    }));
    assert_eq!(wire.status, 7);
    assert_eq!(wire.failure.code, 2);
}
