use super::*;
const GOLDEN: &[u8] = include_bytes!("../../tests/fixtures/action-scale-top-left.bin");
#[test]
fn exact_top_left_request_preserves_center_and_rejects_malformed_payloads() {
    let mut center = fixture("action.6");
    center[56..60].copy_from_slice(&2u32.to_le_bytes());
    assert_eq!(center, GOLDEN);
    let Request::Action(_, EditorAction::ScaleSelection { anchor, factor }) =
        decode(GOLDEN).unwrap().command
    else {
        panic!()
    };
    assert_eq!(anchor, ScaleAnchor::SelectionTopLeft);
    assert_eq!(factor.get(), 1.5);
    let Request::Action(_, EditorAction::ScaleSelection { anchor, .. }) =
        decode(&fixture("action.6")).unwrap().command
    else {
        panic!()
    };
    assert_eq!(anchor, ScaleAnchor::SelectionCenter);
    for end in 0..GOLDEN.len() {
        assert!(decode(&GOLDEN[..end]).is_err());
    }
    let mut tail = GOLDEN.to_vec();
    tail.push(0);
    assert!(decode(&tail).is_err());
    tail[12..16].copy_from_slice(&49u32.to_le_bytes());
    assert!(decode(&tail).is_err());
    for (offset, value) in [(52, 1), (56, 0), (56, 3), (56, u32::MAX), (60, 1)] {
        let mut bad = GOLDEN.to_vec();
        bad[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(decode(&bad).is_err());
    }
}
fn scale_request(a: &mut WireApp, id: u64) -> Vec<u8> {
    let mut p = GOLDEN.to_vec();
    p[24..48].copy_from_slice(&context(a, id));
    p
}
#[test]
fn malformed_scale_does_not_cancel_preview_and_availability_uses_new_anchor() {
    let mut a = app();
    let id = session(&mut a);
    draw(&mut a, id);
    drop(edit(&mut a, id, 1, &[2, 0, 0, 0, 0, 0, 0, 0]));
    drop(edit(&mut a, id, 4, &[1, 0, 0, 0, 0, 0, 0, 0]));
    drop(edit(&mut a, id, 8, &[]));
    let mut available = words(&[id]);
    available.extend_from_slice(&GOLDEN[48..]);
    assert_eq!(
        u32_at(a.call(&packet(4, &available)).unwrap().bytes(), 72),
        1
    );
    let state = a.call(&packet(3, &words(&[id]))).unwrap().bytes().to_vec();
    let original = scale_request(&mut a, id);
    for factor in [0.0, -1.0, 100.0001, f64::NAN, f64::INFINITY] {
        let mut bad = original.clone();
        bad[64..72].copy_from_slice(&factor.to_le_bytes());
        assert!(a.call(&bad).is_err());
        assert_eq!(a.call(&packet(3, &words(&[id]))).unwrap().bytes(), state);
    }
    drop(a.call(&original).unwrap());
    let state = a.call(&packet(3, &words(&[id]))).unwrap();
    assert_eq!(u64_at(state.bytes(), 104), 0);
}
