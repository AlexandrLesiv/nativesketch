use super::*;
#[test]
fn exact_request_goldens_and_invalid_tails() {
    let mut count = 0;
    for (name, bytes) in fixtures() {
        if name.starts_with("tag.") || name.starts_with("failure.") {
            continue;
        }
        let p = decode(&bytes).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(p.request, 0xfedcba9876543210);
        count += 1;
        let mut tail = bytes.clone();
        tail.push(0);
        assert!(decode(&tail).is_err(), "{name}");
        let size = (tail.len() - 24) as u32;
        tail[12..16].copy_from_slice(&size.to_le_bytes());
        assert!(decode(&tail).is_err(), "accepted semantic tail {name}");
        for n in 0..bytes.len() {
            assert!(decode(&bytes[..n]).is_err(), "truncated {name}/{n}");
        }
    }
    assert_eq!(count, 95);
}
#[test]
fn malformed_flags_reserved_versions_counts_and_floats_are_rejected() {
    let mut p = fixture("edit.1");
    p[52] = 1;
    assert_eq!(decode(&p).unwrap_err().failure.code, 5);
    let mut p = fixture("sample.0.1");
    p[68] = 2;
    assert!(decode(&p).is_err());
    let mut p = fixture("sample.0.1");
    p[56..60].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(decode(&p).is_err());
    let mut p = fixture("action.6");
    p[64..72].copy_from_slice(&f64::NAN.to_le_bytes());
    assert!(decode(&p).is_err());
    let mut p = fixture("bootstrap");
    p[4] = 3;
    assert_eq!(decode(&p).unwrap_err().failure.code, 4);
    let mut p = fixture("stats");
    p[8..12].copy_from_slice(&0x80000005u32.to_le_bytes());
    assert!(decode(&p).is_err());
    assert!(matches!(
        decode(&fixture("file.7")).unwrap().command,
        Request::Unsupported
    ));
    assert!(matches!(
        decode(&fixture("file.8")).unwrap().command,
        Request::Unsupported
    ));
}
#[test]
fn sample_kinds_pressure_and_high_ids_are_preserved() {
    for kind in 0..3 {
        for pressure in 0..2 {
            let p = decode(&fixture(&format!("sample.{kind}.{pressure}"))).unwrap();
            let Request::Input(c, PointerId(7), samples) = p.command else {
                panic!()
            };
            assert_eq!(c.session, 0x100000001);
            let (actual_kind, s) = match samples[0] {
                sketch_core::Input::Actual(s) => (0, s),
                sketch_core::Input::Predicted(s) => (1, s),
                sketch_core::Input::Correction(s) => (2, s),
            };
            assert_eq!(actual_kind, kind);
            assert_eq!(s.pressure, if pressure == 1 { Some(0.75) } else { None });
            assert_eq!(s.timestamp, 0.125);
        }
    }
}
