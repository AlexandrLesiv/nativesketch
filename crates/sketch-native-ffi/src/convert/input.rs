use crate::dto;
use sketch_core::{Input, Sample};
pub fn samples(values: &[dto::Sample]) -> Result<Vec<Input>, u32> {
    values
        .iter()
        .map(|v| {
            if v.has_pressure > 1 {
                return Err(1);
            }
            let sample = Sample {
                id: v.sample_id,
                x: v.x,
                y: v.y,
                timestamp: v.timestamp,
                pressure: if v.has_pressure == 1 {
                    Some(v.pressure)
                } else {
                    None
                },
            };
            match v.kind {
                0 => Ok(Input::Actual(sample)),
                1 => Ok(Input::Predicted(sample)),
                2 => Ok(Input::Correction(sample)),
                _ => Err(1),
            }
        })
        .collect()
}
