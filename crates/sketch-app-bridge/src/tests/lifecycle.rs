use super::*;
#[test]
fn worker_finishes_after_actual_owner_exit_and_only_then_result_can_retire() {
    let _serial = serial();
    let (send, receive) = std::sync::mpsc::channel();
    let owner = std::thread::spawn(move || {
        let a = app();
        let s = session(a);
        queued(a, s, 2);
        let w = take(a);
        let meta = crate::registry::meta(a, crate::objects::Kind::App).unwrap();
        send.send((a, w, std::sync::Arc::downgrade(&meta))).unwrap();
    });
    let (a, w, weak) = receive.recv().unwrap();
    owner.join().unwrap();
    assert_eq!(app_call(a, &packet(5, &[])).unwrap_err().status, 7);
    let r = std::thread::spawn(move || finish(w)).join().unwrap();
    result_retire_stopped(r).unwrap();
    assert_eq!(result_retire_stopped(r).unwrap_err().status, 7);
    // Removed app metadata never retains the owner TLS. All its children are gone.
    assert!(!crate::registry::has_children(a));
    assert!(
        weak.upgrade().is_none(),
        "global metadata must not retain terminated TLS"
    );
}
#[test]
fn panic_quarantine_does_not_authorize_result_retirement_until_owner_exits() {
    let _serial = serial();
    let r = std::thread::spawn(|| {
        let a = app();
        let s = session(a);
        queued(a, s, 2);
        let r = finish(take(a));
        let reply = app_call(a, &packet(5, &[])).unwrap();
        let error =
            crate::owner::with_app::<()>(a, |_, _| panic!("test boundary fault")).unwrap_err();
        assert_eq!(error.status, 6);
        assert_eq!(app_call(a, &packet(5, &[])).unwrap_err().status, 10);
        assert_eq!(app_free(a).unwrap_err().status, 10);
        assert_eq!(result_retire_stopped(r).unwrap_err().status, 9);
        buffer_free(reply).unwrap();
        r
    })
    .join()
    .unwrap();
    result_retire_stopped(r).unwrap();
}
