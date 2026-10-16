use super::radius;
use crate::Stroke;
pub(crate) fn hit(s: &Stroke, x: f64, y: f64, tolerance: f64) -> bool {
    let t = s.transform;
    let x = (x - t.tx) / t.scale;
    let y = (y - t.ty) / t.scale;
    let tol = tolerance / t.scale;
    if s.samples
        .iter()
        .any(|p| (x - p.x).hypot(y - p.y) <= radius(s, *p) + tol)
    {
        return true;
    }
    s.samples.windows(2).any(|w| {
        let a = w[0];
        let b = w[1];
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let d = dx * dx + dy * dy;
        if d == 0.0 {
            return false;
        }
        let u = (((x - a.x) * dx + (y - a.y) * dy) / d).clamp(0.0, 1.0);
        (x - a.x - u * dx).hypot(y - a.y - u * dy)
            <= radius(s, a) + (radius(s, b) - radius(s, a)) * u + tol
    })
}
