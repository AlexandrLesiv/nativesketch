mod bounds;
mod hit_test;
mod outline;
use crate::{Sample, Stroke};
pub(crate) use bounds::bounds;
pub(crate) use hit_test::hit;
pub(crate) use outline::render_stroke;
pub(crate) fn radius(s: &Stroke, p: Sample) -> f64 {
    s.brush.width * 0.5 * p.pressure.map_or(1.0, |p| 0.2 + 0.8 * p)
}
