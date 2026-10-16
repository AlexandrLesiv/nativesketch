use super::*;
#[test]
fn output_capacity_does_not_mutate_or_drain_events() {
    let mut app = WireApp::bootstrap(
        &fixture("bootstrap"),
        TransportLimits {
            buffers: 1,
            ..TransportLimits::default()
        },
    )
    .unwrap();
    let r = app.call(&fixture("create.cap3")).unwrap();
    let id = u64_at(r.bytes(), 72);
    assert_eq!(app.call(&fixture("create.cap3")).unwrap_err().status, 3);
    assert_eq!(app.app.stats().sessions, 1);
    assert_eq!(app.call(&packet(7, &[])).unwrap_err().status, 3);
    drop(r);
    assert_eq!(
        app.call(&packet(3, &words(&[id]))).unwrap().bytes().len(),
        224
    );
}
#[test]
fn invalid_trailing_batch_and_stale_context_preserve_active_draft() {
    let mut app = app();
    let id = session(&mut app);
    let mut begin = words(&[7]);
    begin.extend([2, 0, 0, 0, 0, 0, 0, 0]);
    drop(edit(&mut app, id, 6, &begin));
    let before = context(&mut app, id);
    let mut b = before.clone();
    let mut sample = fixture("sample.0.1")[48..].to_vec();
    sample[8..12].copy_from_slice(&2u32.to_le_bytes());
    let mut second = sample[16..].to_vec();
    second[16..24].copy_from_slice(&f64::NAN.to_le_bytes());
    sample.extend(second);
    b.extend(sample);
    assert!(app.call(&packet(12, &b)).is_err());
    assert_eq!(context(&mut app, id), before);
    let mut b = before.clone();
    b[16..24].copy_from_slice(&0u64.to_le_bytes());
    b.extend([1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(app.call(&packet(11, &b)).unwrap_err().status, 7);
    assert_eq!(context(&mut app, id), before);
}
#[test]
fn owner_scope_rejects_worker_and_forged_session_calls() {
    let mut a = app();
    let id = session(&mut a);
    let mut b = app();
    assert_eq!(b.call(&packet(3, &words(&[id]))).unwrap_err().status, 7);
    for op in [0, 30, 31, 100, 101, 102, 103, 106] {
        let p = if op == 0 {
            fixture("bootstrap")
        } else {
            packet(op, &[])
        };
        assert_eq!(a.call(&p).unwrap_err().failure.code, 3);
    }
}
