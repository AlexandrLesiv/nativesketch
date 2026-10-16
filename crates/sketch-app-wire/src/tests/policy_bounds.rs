use super::*;
#[test]
fn explicit_policy_tags_reject_invalid_create_without_consuming_session_or_output_capacity() {
    let mut a = WireApp::bootstrap(
        &fixture("bootstrap"),
        TransportLimits {
            buffers: 1,
            ..TransportLimits::default()
        },
    )
    .unwrap();
    for (offset, value) in [(72, 2u32), (76, 1)] {
        let mut bytes = fixture("create.policy.1");
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(a.call(&bytes).is_err());
        assert_eq!(a.app.stats().sessions, 0);
    }
    let held = a.call(&fixture("create.policy.1")).unwrap();
    let id = u64_at(held.bytes(), 72);
    assert_eq!(a.call(&fixture("create.policy.0")).unwrap_err().status, 3);
    assert_eq!(a.call(&packet(9, &words(&[id]))).unwrap_err().status, 3);
    assert_eq!(a.app.stats().sessions, 1);
    drop(held);
    assert_eq!(
        a.call(&packet(9, &words(&[id]))).unwrap().bytes().len(),
        128
    );
    for (tag, expected) in [
        (0, SaveDestinationPolicy::ReuseBinding),
        (1, SaveDestinationPolicy::AlwaysChoose),
    ] {
        let Request::CreateWithSavePolicy(_, policy) =
            decode(&fixture(&format!("create.policy.{tag}")))
                .unwrap()
                .command
        else {
            panic!()
        };
        assert_eq!(policy, expected);
    }
}
#[test]
fn created_policy_controls_bound_save_while_legacy_create_preserves_default() {
    for (name, chooser) in [
        ("create.cap3", false),
        ("create.policy.0", false),
        ("create.policy.1", true),
    ] {
        let mut a = app();
        let id = u64_at(a.call(&fixture(name)).unwrap().bytes(), 72);
        draw(&mut a, id);
        let op = queued(&mut a, id, 2, 1, 7);
        let work = take(&mut a);
        finish(&mut a, work);
        drop(a.call(&packet(32, &words(&[id, op]))).unwrap());
        let outcome = file(&mut a, id, 1);
        assert_eq!(u32_at(outcome.bytes(), 232), if chooser { 1 } else { 5 });
    }
}
fn bounds(a: &mut WireApp, id: u64) -> Vec<u8> {
    let mut p = fixture("document.bounds");
    p[24..32].copy_from_slice(&id.to_le_bytes());
    a.call(&p).unwrap().bytes().to_vec()
}
fn compare_bounds(a: &mut WireApp, id: u64, name: &str) {
    let mut result = bounds(a, id);
    assert_eq!(
        u64_at(&result, 72),
        u64_at(a.call(&packet(3, &words(&[id]))).unwrap().bytes(), 80)
    );
    result[72..80].copy_from_slice(&0xf123456789abcd20u64.to_le_bytes());
    assert_eq!(result, fixture(name));
}
#[test]
fn bounds_response_goldens_preserve_empty_far_f64_identity_revision_and_readonly_query() {
    let mut a = app();
    let id = session(&mut a);
    compare_bounds(&mut a, id, "response.bounds.empty");
    let mut begin = words(&[7]);
    begin.extend([2, 0, 0, 0, 0, 0, 0, 0]);
    drop(edit(&mut a, id, 6, &begin));
    let mut body = context(&mut a, id);
    let mut samples = fixture("sample.0.0")[48..].to_vec();
    samples[32..40].copy_from_slice(&(900e6_f64 + 0.125).to_le_bytes());
    samples[40..48].copy_from_slice(&(-700e6_f64 - 0.375).to_le_bytes());
    body.extend(samples);
    drop(a.call(&packet(12, &body)).unwrap());
    compare_bounds(&mut a, id, "response.bounds.empty");
    drop(edit(&mut a, id, 7, &words(&[7])));
    compare_bounds(&mut a, id, "response.bounds.far");
    let state = a.app.projection(a.session(id).unwrap()).unwrap();
    let _ = bounds(&mut a, id);
    assert_eq!(a.app.projection(a.session(id).unwrap()).unwrap(), state);
    let mut foreign = app();
    assert_eq!(
        foreign.call(&packet(9, &words(&[id]))).unwrap_err().status,
        7
    );
    let mut create = fixture("create.policy.1");
    create[24..28].fill(0);
    create[28..32].copy_from_slice(&2u32.to_le_bytes());
    let viewer = u64_at(a.call(&create).unwrap().bytes(), 72);
    assert_eq!(u32_at(&bounds(&mut a, viewer), 88), 0);
    drop(a.call(&packet(2, &words(&[viewer]))).unwrap());
    assert!(a.call(&packet(9, &words(&[viewer]))).is_err());
}
