use super::*;
#[test]
fn real_worker_roundtrip_wrong_app_and_double_release_preserve_proof() {
    let _serial = serial();
    let a = app();
    let s = session(a);
    draw(a, s);
    let op = queued(a, s, 2);
    let w = take(a);
    let held = crate::registry::Checkout::take(w, crate::objects::Kind::Work).unwrap();
    std::thread::spawn(move || {
        assert_eq!(work_call(w, &packet(100, &[]), 0).unwrap_err().status, 9)
    })
    .join()
    .unwrap();
    drop(held);
    let r = std::thread::spawn(move || {
        assert_eq!(app_call(a, &packet(5, &[])).unwrap_err().status, 11);
        finish(w)
    })
    .join()
    .unwrap();
    assert_eq!(result_retire_stopped(r).unwrap_err().status, 9);
    let foreign = app();
    assert_eq!(
        app_complete(foreign, &packet(31, &[]), r)
            .unwrap_err()
            .status,
        11
    );
    app_free(foreign).unwrap();
    complete(a, r);
    assert_eq!(app_complete(a, &packet(31, &[]), r).unwrap_err().status, 7);
    assert_eq!(u32_at(&call(a, &packet(32, &words(&[s, op]))), 156) & 1, 0);
    close(a, s);
    assert_eq!(app_free(a).unwrap_err().status, 7);
}
#[test]
fn owner_reentrancy_and_same_object_concurrency_reject_without_consumption() {
    let _serial = serial();
    let a = app();
    crate::owner::with_app(a, |_, _| {
        assert_eq!(app_call(a, &packet(5, &[])).unwrap_err().status, 9);
        Ok(())
    })
    .unwrap();
    let r = app_call(a, &packet(5, &[])).unwrap();
    let held = crate::registry::Checkout::take(r, crate::objects::Kind::Buffer).unwrap();
    std::thread::spawn(move || assert_eq!(buffer_free(r).unwrap_err().status, 9))
        .join()
        .unwrap();
    drop(held);
    assert!(buffer_len(r).unwrap() > 72);
    buffer_free(r).unwrap();
    app_free(a).unwrap();
}
#[test]
fn bad_packets_wrong_kind_and_byte_ranges_leave_buffers_usable() {
    let _serial = serial();
    let a = app();
    assert_eq!(app_call(a, b"broken").unwrap_err().status, 1);
    let r = app_call(a, &packet(5, &[])).unwrap();
    assert_eq!(input_free(r).unwrap_err().status, 1);
    let mut bytes = [91; 8];
    let len = buffer_len(r).unwrap();
    assert_eq!(buffer_copy(r, len + 1, &mut bytes).unwrap_err().status, 1);
    assert_eq!(bytes, [91; 8]);
    assert_eq!(buffer_copy(r, len, &mut bytes).unwrap(), 0);
    assert_eq!(app_free(a).unwrap_err().status, 9);
    buffer_free(r).unwrap();
    assert_eq!(buffer_free(r).unwrap_err().status, 7);
    app_free(a).unwrap();
}
