use super::*;
#[test]
fn cancelled_queued_loads_do_not_exhaust_operation_registry() {
    let mut a = app();
    let id = session(&mut a);
    for resource in 1..100 {
        let op = queued(&mut a, id, 5, resource, 1);
        drop(a.call(&packet(33, &words(&[op]))).unwrap());
        assert!(a.operations.is_empty());
        drop(a.call(&packet(34, &[])).unwrap());
    }
    assert_eq!(a.app.stats().operations, 0);
}
#[test]
fn cancelled_actual_write_keeps_receipt_and_shutdown_drains() {
    let mut a = app();
    let id = session(&mut a);
    draw(&mut a, id);
    let op = queued(&mut a, id, 2, 1, 7);
    let work = take(&mut a);
    drop(a.call(&packet(33, &words(&[op]))).unwrap());
    assert!(!a.shutdown_ready());
    finish(&mut a, work);
    drop(a.call(&packet(32, &words(&[id, op]))).unwrap());
    let r = file(&mut a, id, 6);
    let token = u64_at(r.bytes(), 216);
    drop(r);
    let mut b = context(&mut a, id);
    b.extend(words(&[token]));
    b.extend([3, 0, 0, 0, 0, 0, 0, 0]);
    let r = a.call(&packet(21, &b)).unwrap();
    let token = u64_at(r.bytes(), 216);
    drop(r);
    let mut b = context(&mut a, id);
    b.extend(words(&[token]));
    b.extend([5, 0, 0, 0, 0, 0, 0, 0]);
    drop(a.call(&packet(21, &b)).unwrap());
    drop(a.call(&packet(2, &words(&[id]))).unwrap());
    drain(&mut a);
    drop(a.call(&packet(34, &[])).unwrap());
    assert!(a.shutdown_ready());
}
