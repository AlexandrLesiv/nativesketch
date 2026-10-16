use crate::*;
pub(super) fn brush() -> Brush {
    Brush {
        rgba: [0.2, 0.3, 0.4, 0.5],
        width: 10.0,
    }
}
pub(super) fn sample(id: u64, x: f64, pressure: Option<f64>) -> Sample {
    Sample {
        id,
        x,
        y: 10.0,
        timestamp: id as f64 / 100.0,
        pressure,
    }
}
pub(super) fn draw(e: &mut Editor) {
    e.begin(7, 1, brush()).unwrap();
    e.append(
        7,
        &[
            Input::Actual(sample(1, 0.0, None)),
            Input::Actual(sample(2, 10.0, Some(0.5))),
            Input::Actual(sample(3, 20.0, Some(1.0))),
        ],
    )
    .unwrap();
    e.commit(7).unwrap();
}

mod editing;
mod input;
mod persistence;
mod properties;
