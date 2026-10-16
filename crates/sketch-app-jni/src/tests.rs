use crate::{
    boundary::guarded,
    direct::*,
    receipt::{Receipt, error},
};
#[test]
fn fixed_receipt_preserves_high_bits_zeros_and_consumption_fields() {
    let r = Receipt {
        flags: 3,
        reply: 0xf123456789abcdef,
        result: 0xe123456789abcdef,
        ..Receipt::default()
    };
    let (status, b) = Receipt::bytes(Ok(r), (11, 27));
    assert_eq!(status, 0);
    assert_eq!(&b[..8], &[0, 0, 0, 0, 3, 0, 0, 0]);
    assert_eq!(&b[8..16], &[0; 8]);
    assert_eq!(&b[16..24], &0xf123456789abcdefu64.to_le_bytes());
    assert_eq!(&b[24..40], &[0; 16]);
    assert_eq!(&b[40..48], &0xe123456789abcdefu64.to_le_bytes());
    assert_eq!(&b[48..], &[0; 80]);
}
#[test]
fn ordinary_rejection_publishes_zero_ownership_and_correlated_wire_failure() {
    let (status, b) = Receipt::bytes(Err(error(11)), (0x70000005, 0xfedcba9876543210));
    assert_eq!(status, 11);
    assert_eq!(&b[..4], &11u32.to_le_bytes());
    assert_eq!(&b[4..56], &[0; 52]);
    assert_eq!(&b[56..60], b"NSAP");
    assert_eq!(&b[64..68], &0xf0000005u32.to_le_bytes());
    assert_eq!(&b[72..80], &0xfedcba9876543210u64.to_le_bytes());
    assert_eq!(&b[80..84], &11u32.to_le_bytes());
}
#[test]
fn extent_preflight_rejects_negative_oversized_and_overflowing_ranges() {
    assert_eq!(length_checked(-1, 10).unwrap_err().status, 1);
    assert_eq!(length_checked(11, 10).unwrap_err().status, 3);
    assert_eq!(length_checked(0, 0).unwrap(), 0);
    assert!(checked_extent(0, 0).is_err());
    assert!(checked_extent(usize::MAX - 2, 3).is_err());
    assert!(checked_extent(1, isize::MAX as usize + 1).is_err());
    assert!(checked_extent(8, 16).is_ok());
    assert!(overlap(8, 16, 16, 16));
    assert!(overlap(16, 16, 8, 16));
    assert!(!overlap(8, 8, 16, 16));
    assert!(!overlap(8, 0, 8, 16));
}
#[test]
fn boundary_panic_is_terminal_without_crossing_the_export() {
    let failure = guarded(|| panic!("synthetic panic before any bridge transfer"));
    let (status, bytes) = Receipt::bytes(failure, (11, 17));
    assert_eq!(status, 6);
    assert_eq!(&bytes[4..56], &[0; 52]);
    assert_eq!(&bytes[80..84], &6u32.to_le_bytes());
}
