use super::*;
#[test]
fn cancelling_bound_save_while_load_pending_keeps_issued_save_completion() {
    let mut config = fixture("bootstrap");
    config[32..40].copy_from_slice(&2u64.to_le_bytes());
    let mut a = WireApp::bootstrap(&config, TransportLimits::default()).unwrap();
    let id = session(&mut a);
    draw(&mut a, id);
    let op = queued(&mut a, id, 2, 1, 7);
    let work = take(&mut a);
    finish(&mut a, work);
    drop(a.call(&packet(32, &words(&[id, op]))).unwrap());
    for resource in 2..22 {
        let load = queued(&mut a, id, 5, resource, 1);
        let r = file(&mut a, id, 1);
        assert_eq!(u32_at(r.bytes(), 160), 5);
        let save = u64_at(r.bytes(), 248);
        drop(r);
        drop(a.call(&packet(33, &words(&[save]))).unwrap());
        let r = a
            .call(&packet(32, &words(&[id, save])))
            .expect("cancelled save receipt remains applicable");
        assert_eq!(u32_at(r.bytes(), 312), 4);
        drop(r);
        drop(a.call(&packet(33, &words(&[load]))).unwrap());
        let r = a.call(&packet(34, &[])).unwrap();
        assert_eq!(u32_at(r.bytes(), 72), 1);
        assert_eq!(u64_at(r.bytes(), 96), resource);
        drop(r);
        assert_eq!(a.app.stats().outstanding_saves, 0);
        assert_eq!(a.app.stats().resource_leases, 1);
        assert!(a.operations.is_empty());
    }
    let r = file(&mut a, id, 6);
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
