use super::{MAX_BYTES, reader::Reader};
use crate::geometry::bounds;
use crate::model::coordinate;
use crate::{Brush, Document, Error, MAX_SAMPLES, MAX_STROKES, Result, Sample, Stroke, Transform};
pub(super) const DOCUMENT_BYTES: usize = 24;
pub(super) const STROKE_BYTES: usize = 96;
pub(super) const SAMPLE_BYTES: usize = 48;
const MAGIC: &[u8; 8] = b"NSKETCH1";
fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}
fn put(bytes: &mut Vec<u8>, n: u64) {
    bytes.extend_from_slice(&n.to_le_bytes());
}
pub(crate) fn encode(document: &Document) -> Vec<u8> {
    let mut b = MAGIC.to_vec();
    put(&mut b, document.strokes.len() as u64);
    for s in &document.strokes {
        put(&mut b, s.id);
        put(&mut b, u64::from(s.device));
        put(&mut b, s.pointer_id);
        for x in s.brush.rgba.into_iter().chain([
            s.brush.width,
            s.transform.scale,
            s.transform.tx,
            s.transform.ty,
        ]) {
            put(&mut b, x.to_bits());
        }
        put(&mut b, s.samples.len() as u64);
        for p in &s.samples {
            put(&mut b, p.id);
            for x in [p.x, p.y, p.timestamp] {
                put(&mut b, x.to_bits());
            }
            put(&mut b, u64::from(p.pressure.is_some()));
            put(&mut b, p.pressure.unwrap_or(0.0).to_bits());
        }
    }
    let hash = checksum(&b);
    put(&mut b, hash);
    b
}
pub(crate) fn decode(bytes: &[u8]) -> Result<Document> {
    if bytes.len() as u64 > MAX_BYTES || bytes.len() < 24 || &bytes[..8] != MAGIC {
        return Err(Error::Format);
    }
    let end = bytes.len() - 8;
    let expected = u64::from_le_bytes(bytes[end..].try_into().map_err(|_| Error::Format)?);
    if checksum(&bytes[..end]) != expected {
        return Err(Error::Format);
    }
    let mut r = Reader {
        data: &bytes[..end],
        pos: 8,
    };
    let count = r.u64()?;
    if count > MAX_STROKES as u64 {
        return Err(Error::Format);
    }
    let mut strokes = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for _ in 0..count {
        let id = r.u64()?;
        if id == 0 || id == u64::MAX || !ids.insert(id) {
            return Err(Error::Format);
        }
        let device = r.u64()?;
        if device > 2 {
            return Err(Error::Format);
        }
        let pointer_id = r.u64()?;
        let brush = Brush {
            rgba: [r.f64()?, r.f64()?, r.f64()?, r.f64()?],
            width: r.f64()?,
        };
        brush.validate().map_err(|_| Error::Format)?;
        let transform = Transform {
            scale: r.f64()?,
            tx: r.f64()?,
            ty: r.f64()?,
        };
        transform.validate().map_err(|_| Error::Format)?;
        let n = r.u64()?;
        if n == 0 || n > MAX_SAMPLES as u64 {
            return Err(Error::Format);
        }
        let mut samples: Vec<Sample> = Vec::new();
        for _ in 0..n {
            let id = r.u64()?;
            let x = r.f64()?;
            let y = r.f64()?;
            let timestamp = r.f64()?;
            let flag = r.u64()?;
            let p = r.f64()?;
            if flag > 1 || !p.is_finite() || flag == 0 && p != 0.0 {
                return Err(Error::Format);
            }
            let sample = Sample {
                id,
                x,
                y,
                timestamp,
                pressure: if flag == 1 { Some(p) } else { None },
            };
            sample.validate().map_err(|_| Error::Format)?;
            if samples
                .last()
                .is_some_and(|s| sample.id <= s.id || sample.timestamp < s.timestamp)
            {
                return Err(Error::Format);
            }
            samples.push(sample);
        }
        let stroke = Stroke {
            id,
            device: device as u32,
            pointer_id,
            brush,
            transform,
            samples,
        };
        let b = bounds(&stroke, transform);
        if !coordinate(b.x) || !coordinate(b.y) || !coordinate(b.width) || !coordinate(b.height) {
            return Err(Error::Format);
        }
        strokes.push(stroke);
    }
    if r.pos != end {
        return Err(Error::Format);
    }
    Ok(Document { strokes })
}
