use super::*;
fn query(app: &mut WireApp, id: u64, previous: u64, viewport: [f64; 5]) -> Reply {
    let mut body = words(&[id, previous]);
    for v in viewport {
        body.extend(v.to_le_bytes());
    }
    app.call(&packet(40, &body)).unwrap()
}
const VIEW: [f64; 5] = [-100.0, -100.0, 400.0, 400.0, 2.0];
#[test]
fn same_frame_borrows_lease_and_live_frames_reuse_committed_root() {
    let mut a = app();
    let id = session(&mut a);
    draw(&mut a, id);
    let r = query(&mut a, id, 0, VIEW);
    let first = u64_at(r.bytes(), 72);
    let root = u64_at(r.bytes(), 104);
    drop(r);
    let r = query(&mut a, id, first, VIEW);
    assert_eq!(u64_at(r.bytes(), 72), first);
    assert_eq!(u32_at(r.bytes(), 120), 1);
    drop(r);
    assert_eq!(a.scenes.leases.len(), 1);
    let mut begin = words(&[7]);
    begin.extend([2, 0, 0, 0, 0, 0, 0, 0]);
    drop(edit(&mut a, id, 6, &begin));
    let mut previous = first;
    for sample in 1..20 {
        let mut b = context(&mut a, id);
        let mut s = fixture("sample.0.1")[48..].to_vec();
        s[24..32].copy_from_slice(&(sample as u64).to_le_bytes());
        s[48..56].copy_from_slice(&(f64::from(sample)).to_le_bytes());
        b.extend(s);
        drop(a.call(&packet(12, &b)).unwrap());
        let r = query(&mut a, id, previous, VIEW);
        let current = u64_at(r.bytes(), 72);
        assert_ne!(current, previous);
        assert_eq!(u64_at(r.bytes(), 104), root);
        assert_eq!(u32_at(r.bytes(), 124), 1);
        drop(r);
        assert_eq!(a.scenes.cache.stats().committed_builds, 1);
        drop(a.call(&packet(44, &words(&[previous]))).unwrap());
        previous = current;
        assert_eq!(a.scenes.roots.len(), 1);
        assert_eq!(a.scenes.geometries.len(), 2);
    }
    drop(a.call(&packet(44, &words(&[previous]))).unwrap());
    assert_eq!(a.scenes.bytes, 0);
    assert!(a.scenes.geometries.is_empty());
}
#[test]
fn page_boundaries_wrong_membership_and_failed_query_keep_old_scene() {
    let mut a = app();
    let id = session(&mut a);
    draw(&mut a, id);
    let r = query(&mut a, id, 0, VIEW);
    let scene = u64_at(r.bytes(), 72);
    drop(r);
    let page = |scene, first: u32, limit: u32| {
        let mut b = words(&[scene]);
        b.extend(first.to_le_bytes());
        b.extend(limit.to_le_bytes());
        packet(41, &b)
    };
    let r = a.call(&page(scene, 0, u32::MAX)).unwrap();
    assert_eq!(u32_at(r.bytes(), 72), 1);
    let geometry = u64_at(r.bytes(), 96);
    drop(r);
    assert_eq!(u32_at(a.call(&page(scene, 0, 0)).unwrap().bytes(), 72), 0);
    assert_eq!(u32_at(a.call(&page(scene, 1, 1)).unwrap().bytes(), 72), 0);
    assert!(a.call(&page(scene, 2, 1)).is_err());
    let r = a.call(&packet(42, &words(&[scene, geometry]))).unwrap();
    let commands = u64_at(r.bytes(), 120);
    drop(r);
    let mut b = words(&[scene, geometry]);
    b.extend([0, 0, 0, 0, 255, 255, 255, 255]);
    let r = a.call(&packet(43, &b)).unwrap();
    assert_eq!(u32_at(r.bytes(), 72) as u64, commands);
    assert!(r.bytes().len() <= 65536 + 80);
    drop(r);
    let other = session(&mut a);
    let r = query(&mut a, other, 0, VIEW);
    let other_scene = u64_at(r.bytes(), 72);
    drop(r);
    assert!(
        a.call(&packet(42, &words(&[other_scene, geometry])))
            .is_err()
    );
    let mut b = words(&[id, scene]);
    for v in [-100.0f64, -100.0, 0.0, 400.0, 2.0] {
        b.extend(v.to_le_bytes());
    }
    assert!(a.call(&packet(40, &b)).is_err());
    assert!(a.call(&page(scene, 0, 1)).is_ok());
    let mut foreign = app();
    assert!(foreign.call(&page(scene, 0, 1)).is_err());
}
#[test]
fn scene_capacity_and_pin_budget_fail_before_publication() {
    let mut a = WireApp::bootstrap(
        &fixture("bootstrap"),
        TransportLimits {
            scenes: 1,
            ..TransportLimits::default()
        },
    )
    .unwrap();
    let id = session(&mut a);
    draw(&mut a, id);
    let r = query(&mut a, id, 0, VIEW);
    let scene = u64_at(r.bytes(), 72);
    drop(r);
    draw(&mut a, id);
    let mut b = words(&[id, scene]);
    for v in VIEW {
        b.extend(v.to_le_bytes());
    }
    assert_eq!(a.call(&packet(40, &b)).unwrap_err().status, 3);
    assert_eq!(a.scenes.leases.len(), 1);
    drop(a.call(&packet(44, &words(&[scene]))).unwrap());
    assert!(a.call(&packet(44, &words(&[scene]))).is_err());
    a.scenes.limits.scene_bytes = 1;
    let mut b = words(&[id, 0]);
    for v in VIEW {
        b.extend(v.to_le_bytes());
    }
    assert_eq!(a.call(&packet(40, &b)).unwrap_err().status, 3);
    assert!(a.scenes.leases.is_empty());
    assert_eq!(a.scenes.bytes, 0);
}
