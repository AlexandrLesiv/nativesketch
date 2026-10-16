use super::codec::{DOCUMENT_BYTES, SAMPLE_BYTES, STROKE_BYTES};
use crate::{Document, Sample, Stroke};
pub(crate) fn encoded_bytes(document: &Document) -> Option<usize> {
    sizes(
        document.strokes.len(),
        document.strokes.iter().map(|s| s.samples.len()),
    )
    .map(|(_, encoded)| encoded)
}
/// Checked payload accounting, excluding allocator bookkeeping and worker encoded bytes.
/// Reads stroke headers/sample lengths only; performs no sample traversal or encoding.
pub(crate) fn snapshot_bytes(document: &Document) -> Option<usize> {
    payload_bytes(
        document.strokes.len(),
        document.strokes.iter().map(|s| s.samples.len()),
    )
}
fn payload_bytes(strokes: usize, samples: impl Iterator<Item = usize>) -> Option<usize> {
    sizes(strokes, samples).map(|(memory, _)| memory)
}
fn sizes(strokes: usize, samples: impl Iterator<Item = usize>) -> Option<(usize, usize)> {
    let mut encoded = DOCUMENT_BYTES;
    let mut memory =
        size_of::<Document>().checked_add(strokes.checked_mul(size_of::<Stroke>())?)?;
    for count in samples {
        encoded = encoded
            .checked_add(STROKE_BYTES)?
            .checked_add(count.checked_mul(SAMPLE_BYTES)?)?;
        if encoded as u64 > super::MAX_BYTES {
            return None;
        }
        memory = memory.checked_add(count.checked_mul(size_of::<Sample>())?)?;
    }
    Some((memory, encoded))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encoded_ceiling_and_accounting_overflow_fail_before_allocation() {
        let boundary = (super::super::MAX_BYTES as usize - 24 - 96) / 48;
        assert!(payload_bytes(1, [boundary].into_iter()).is_some());
        assert_eq!(payload_bytes(1, [boundary + 1].into_iter()), None);
        assert_eq!(payload_bytes(1, [usize::MAX].into_iter()), None);
        assert_eq!(payload_bytes(usize::MAX, [].into_iter()), None);
    }
    #[test]
    fn frozen_document_estimate_matches_exact_payload_sizes() {
        let bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/portable-v1.nsketch"
        ));
        let document = super::super::decode(bytes).unwrap();
        let samples: usize = document.strokes.iter().map(|s| s.samples.len()).sum();
        assert_eq!(bytes.len(), 24 + 96 * document.strokes.len() + 48 * samples);
        assert_eq!(
            snapshot_bytes(&document),
            Some(
                size_of::<Document>()
                    + size_of::<Stroke>() * document.strokes.len()
                    + size_of::<Sample>() * samples
            )
        );
    }
}
