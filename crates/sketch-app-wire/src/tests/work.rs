use super::*;
#[test]
fn issued_write_crosses_thread_and_wrong_owner_retains_result() {
    let mut a = app();
    let id = session(&mut a);
    draw(&mut a, id);
    let op = queued(&mut a, id, 2, 1, 7);
    let work = take(&mut a);
    let mut result = std::thread::spawn(move || {
        let mut work = work;
        drop(work.call(&packet(101, &[]), &mut None).unwrap());
        assert_eq!(
            work.call(&packet(104, &[4, 0, 0, 0, 0, 0, 0, 0]), &mut None)
                .unwrap_err()
                .status,
            8
        );
        let r = work
            .call(
                &packet(107, &[0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0]),
                &mut None,
            )
            .unwrap();
        assert!(u32_at(r.reply.bytes(), 72) > 24);
        drop(r);
        work.call(&packet(104, &[1, 0, 0, 0, 3, 0, 0, 0]), &mut None)
            .unwrap()
            .result
    })
    .join()
    .unwrap();
    let mut foreign = app();
    assert_eq!(
        foreign
            .complete(&packet(31, &[]), &mut result)
            .unwrap_err()
            .status,
        11
    );
    assert!(result.is_some());
    drop(a.complete(&packet(31, &[]), &mut result).unwrap());
    assert!(result.is_none());
    let r = a.call(&packet(32, &words(&[id, op]))).unwrap();
    assert_eq!(r.bytes().len(), 376);
    assert_eq!(u32_at(r.bytes(), 164) & 1, 0);
}
#[test]
fn completed_aliases_release_once_and_keep_one_bound_lease() {
    let mut a = app();
    let id = session(&mut a);
    draw(&mut a, id);
    for resource in 1..8 {
        let op = queued(&mut a, id, if resource == 1 { 2 } else { 3 }, resource, 7);
        let work = take(&mut a);
        finish(&mut a, work);
        drop(a.call(&packet(32, &words(&[id, op]))).unwrap());
        let r = a.call(&packet(34, &[])).unwrap();
        assert_eq!(u32_at(r.bytes(), 72), u32::from(resource != 1));
        if resource != 1 {
            assert_eq!(u64_at(r.bytes(), 96), resource);
        }
        drop(r);
        assert_eq!(u32_at(a.call(&packet(34, &[])).unwrap().bytes(), 72), 0);
        assert_eq!(a.app.stats().resource_leases, 1);
    }
}
