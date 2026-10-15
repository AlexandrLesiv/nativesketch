pub(crate) fn put64(b: &mut Vec<u8>, x: u64) {
    b.extend_from_slice(&x.to_le_bytes());
}
pub(crate) fn put32(b: &mut Vec<u8>, x: u32) {
    b.extend_from_slice(&x.to_le_bytes());
}
pub(crate) fn doubles(b: &mut Vec<u8>, values: impl IntoIterator<Item = f64>) {
    for x in values {
        put64(b, x.to_bits());
    }
}
