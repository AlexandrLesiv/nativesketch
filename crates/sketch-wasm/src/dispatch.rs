use crate::encoding::{doubles, put32, put64};
use crate::reader::Reader;
use sketch_core::{Brush, Editor, Error, Input, Sample};
pub(crate) fn command(e: &mut Editor, op: u32, bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let mut r = Reader(bytes);
    let mut out = Vec::new();
    // Validate fixed message length before mutations; no trailing garbage accepted.
    let length = match op {
        1 | 5 | 6 | 9 | 10 | 11 | 13 | 14 | 15 | 16 | 17 => Some(0),
        2 => Some(52),
        4 | 19 => Some(8),
        7 | 12 => Some(24),
        8 => Some(4),
        3 | 18 => None,
        _ => return Err(Error::InvalidArgument),
    };
    if length.is_some_and(|n| n != bytes.len()) {
        return Err(Error::InvalidArgument);
    }
    match op {
        1 => {
            let s = e.state();
            put64(&mut out, s.count as u64);
            put64(&mut out, s.selected);
            for n in [
                u32::from(s.can_undo),
                u32::from(s.can_redo),
                u32::from(s.dirty),
                s.interaction,
            ] {
                put32(&mut out, n);
            }
            doubles(
                &mut out,
                [s.bounds.x, s.bounds.y, s.bounds.width, s.bounds.height],
            );
        }
        2 => {
            let pointer = r.u64()?;
            let device = r.u32()?;
            let brush = Brush {
                rgba: [r.f64()?, r.f64()?, r.f64()?, r.f64()?],
                width: r.f64()?,
            };
            e.begin(pointer, device, brush)?;
        }
        3 => {
            let pointer = r.u64()?;
            if r.0.len() % 48 != 0 || r.0.len() / 48 > sketch_core::MAX_SAMPLES {
                return Err(Error::InvalidArgument);
            }
            let mut inputs = Vec::new();
            while !r.0.is_empty() {
                let x = r.f64()?;
                let y = r.f64()?;
                let timestamp = r.f64()?;
                let pressure = r.f64()?;
                let id = r.u64()?;
                let flag = r.u32()?;
                let kind = r.u32()?;
                if flag > 1 || !pressure.is_finite() {
                    return Err(Error::InvalidArgument);
                }
                let s = Sample {
                    x,
                    y,
                    timestamp,
                    id,
                    pressure: if flag == 1 { Some(pressure) } else { None },
                };
                inputs.push(match kind {
                    0 => Input::Actual(s),
                    1 => Input::Predicted(s),
                    2 => Input::Correction(s),
                    _ => return Err(Error::InvalidArgument),
                });
            }
            e.append(pointer, &inputs)?;
        }
        4 => e.commit(r.u64()?)?,
        5 => e.cancel(),
        6 => {
            for c in e.render() {
                put64(&mut out, c.stroke_id);
                put32(&mut out, c.verb);
                put32(&mut out, 0);
                doubles(&mut out, c.rgba);
                doubles(&mut out, c.points);
            }
        }
        7 => e.select_at(r.f64()?, r.f64()?, r.f64()?)?,
        8 => e.select_cycle(r.u32()? as i32)?,
        9 => e.select_clear()?,
        10 => e.delete_selected()?,
        11 => e.resize_begin()?,
        12 => e.resize_update(r.f64()?, r.f64()?, r.f64()?)?,
        13 => e.resize_commit()?,
        14 => e.undo()?,
        15 => e.redo()?,
        16 => e.clear()?,
        17 => {
            let (bytes, revision) = e.export()?;
            put64(&mut out, revision);
            out.extend_from_slice(&bytes);
        }
        18 => {
            e.import(r.0)?;
            r.0 = &[];
        }
        19 => e.mark_saved(r.u64()?)?,
        _ => unreachable!(),
    }
    r.done()?;
    Ok(out)
}
