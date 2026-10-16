use sketch_core::{Input, Sample};

pub fn number(byte: u8) -> f64 {
    match byte {
        252 => f64::NAN,
        253 => f64::INFINITY,
        254 => -f64::INFINITY,
        255 => 1e100,
        _ => f64::from(byte) - 120.0,
    }
}

pub fn batch(bytes: &[u8], step: u64) -> [Input; 2] {
    let sample = Sample {
        id: if bytes[1] & 4 == 0 { step + 1 } else { 1 },
        x: number(bytes[2]),
        y: number(bytes[3]),
        timestamp: step as f64,
        pressure: (bytes[1] & 8 != 0).then_some(number(bytes[4]) / 120.0),
    };
    let next = Sample {
        id: sample.id + 1,
        x: number(bytes[5]),
        ..sample
    };
    [
        if bytes[1] & 16 == 0 {
            Input::Actual(sample)
        } else {
            Input::Correction(sample)
        },
        if bytes[1] & 32 == 0 {
            Input::Predicted(next)
        } else {
            Input::Actual(next)
        },
    ]
}
