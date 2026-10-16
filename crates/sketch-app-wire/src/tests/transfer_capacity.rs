use super::*;
#[test]
fn consumed_events_and_work_wait_for_output_capacity() {
    let mut a = WireApp::bootstrap(
        &fixture("bootstrap"),
        TransportLimits {
            buffers: 1,
            ..TransportLimits::default()
        },
    )
    .unwrap();
    let id = session(&mut a);
    draw(&mut a, id);
    let op = queued(&mut a, id, 3, 1, 7);
    let held = a.call(&packet(5, &[])).unwrap();
    assert_eq!(a.take_work(&packet(30, &[])).unwrap_err().status, 3);
    assert_eq!(a.app.stats().buffers, 0);
    drop(held);
    let mut work = take(&mut a);
    drop(work.call(&packet(101, &[]), &mut None).unwrap());
    let held = a.call(&packet(5, &[])).unwrap();
    assert_eq!(
        work.call(&packet(104, &[1, 0, 0, 0, 3, 0, 0, 0]), &mut None)
            .unwrap_err()
            .status,
        3
    );
    assert!(work.info().is_ok());
    drop(held);
    let mut result = work
        .call(&packet(104, &[1, 0, 0, 0, 3, 0, 0, 0]), &mut None)
        .unwrap()
        .result;
    let held = a.call(&packet(5, &[])).unwrap();
    assert_eq!(
        a.complete(&packet(31, &[]), &mut result)
            .unwrap_err()
            .status,
        3
    );
    assert!(result.is_some());
    drop(held);
    drop(a.complete(&packet(31, &[]), &mut result).unwrap());
    drop(a.call(&packet(32, &words(&[id, op]))).unwrap());
    let held = a.call(&packet(5, &[])).unwrap();
    assert_eq!(a.call(&packet(7, &[])).unwrap_err().status, 3);
    assert_eq!(a.call(&packet(34, &[])).unwrap_err().status, 3);
    drop(held);
    assert_eq!(u32_at(a.call(&packet(7, &[])).unwrap().bytes(), 72), 1);
    assert_eq!(u32_at(a.call(&packet(34, &[])).unwrap().bytes(), 72), 1);
    assert_eq!(u32_at(a.call(&packet(34, &[])).unwrap().bytes(), 72), 0);
}
#[test]
fn wrong_worker_kind_and_foreign_input_preserve_both_objects() {
    let mut a = app();
    let id = session(&mut a);
    let _ = queued(&mut a, id, 5, 1, 1);
    let mut work = take(&mut a);
    let mut b = app();
    let id = session(&mut b);
    let _ = queued(&mut b, id, 5, 1, 1);
    let foreign = take(&mut b);
    let mut input = Some(foreign.copy_input(b"corrupt").unwrap());
    assert_eq!(
        work.call(&packet(102, &[]), &mut input).unwrap_err().status,
        11
    );
    assert!(input.is_some());
    assert!(work.info().is_ok());
    assert!(work.call(&packet(101, &[]), &mut input).is_err());
    assert!(input.is_some());
    input = Some(work.copy_input(b"corrupt").unwrap());
    let mut result = work.call(&packet(102, &[]), &mut input).unwrap().result;
    assert!(input.is_none());
    assert!(work.info().is_err());
    drop(a.complete(&packet(31, &[]), &mut result).unwrap());
}
