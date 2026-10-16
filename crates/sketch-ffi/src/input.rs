use crate::handles::edit;
use crate::{SketchBrush, SketchEditor, SketchSample};
use sketch_core::{Brush, Error, Input, Sample};
/// # Safety
/// Handle must be live with serialized access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_stroke_begin(
    e: *mut SketchEditor,
    pointer: u64,
    device: u32,
    b: SketchBrush,
) -> u32 {
    unsafe {
        edit(e, |e| {
            e.begin(
                pointer,
                device,
                Brush {
                    rgba: [b.red, b.green, b.blue, b.alpha],
                    width: b.width,
                },
            )
        })
    }
}
/// # Safety
/// Handle must be live/serialized. `samples` must address `count` initialized samples,
/// borrowed for this call, or may be null when count is zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_stroke_append(
    e: *mut SketchEditor,
    pointer: u64,
    samples: *const SketchSample,
    count: usize,
) -> u32 {
    unsafe {
        edit(e, |e| {
            if count > sketch_core::MAX_SAMPLES || count > 0 && samples.is_null() {
                return Err(Error::InvalidArgument);
            }
            let slice = if count == 0 {
                &[]
            } else {
                std::slice::from_raw_parts(samples, count)
            };
            let inputs: sketch_core::Result<Vec<_>> = slice
                .iter()
                .map(|s| {
                    if s.has_pressure > 1 || !s.pressure.is_finite() {
                        return Err(Error::InvalidArgument);
                    }
                    let p = Sample {
                        x: s.x,
                        y: s.y,
                        timestamp: s.timestamp,
                        pressure: if s.has_pressure == 1 {
                            Some(s.pressure)
                        } else {
                            None
                        },
                        id: s.sample_id,
                    };
                    match s.kind {
                        0 => Ok(Input::Actual(p)),
                        1 => Ok(Input::Predicted(p)),
                        2 => Ok(Input::Correction(p)),
                        _ => Err(Error::InvalidArgument),
                    }
                })
                .collect();
            e.append(pointer, &inputs?)
        })
    }
}
/// # Safety
/// Handle must be live with serialized access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_stroke_commit(e: *mut SketchEditor, pointer: u64) -> u32 {
    unsafe { edit(e, |e| e.commit(pointer)) }
}
