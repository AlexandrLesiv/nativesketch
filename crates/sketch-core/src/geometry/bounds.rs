use super::radius;
use crate::{Bounds, Stroke, Transform};
pub(crate) fn bounds(s: &Stroke, t: Transform) -> Bounds {
    let mut low = [f64::INFINITY; 2];
    let mut high = [f64::NEG_INFINITY; 2];
    for p in &s.samples {
        let (x, y) = t.point(p.x, p.y);
        let r = radius(s, *p) * t.scale;
        low[0] = low[0].min(x - r);
        low[1] = low[1].min(y - r);
        high[0] = high[0].max(x + r);
        high[1] = high[1].max(y + r);
    }
    Bounds {
        x: low[0],
        y: low[1],
        width: high[0] - low[0],
        height: high[1] - low[1],
    }
}
