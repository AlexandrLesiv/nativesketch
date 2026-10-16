use super::*;
#[test]
fn fixed_errors_correlate_only_exact_v2_frames_without_narrowing() {
    let mut p = packet(0x70000005, &[]);
    p[16..24].copy_from_slice(&0xfedcba9876543210u64.to_le_bytes());
    assert_eq!(correlation(&p), (0x70000005, 0xfedcba9876543210));
    for i in 0..24 {
        assert_eq!(correlation(&p[..i]), (0, 0));
    }
    p.push(0);
    assert_eq!(correlation(&p), (0, 0));
    p.pop();
    p[6] = 1;
    assert_eq!(correlation(&p), (0, 0));
}
