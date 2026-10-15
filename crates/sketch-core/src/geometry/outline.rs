use super::radius;
use crate::{PathCommand, Sample, Stroke, Transform};
pub(crate) fn render_stroke(
    s: &Stroke,
    t: Transform,
    samples: impl Iterator<Item = Sample>,
    out: &mut Vec<PathCommand>,
) {
    let mut previous = None;
    let mut emit = |verb, points| {
        out.push(PathCommand {
            stroke_id: s.id,
            rgba: s.brush.rgba,
            verb,
            points,
        })
    };
    for sample in samples {
        let (x, y) = t.point(sample.x, sample.y);
        let r = radius(s, sample) * t.scale;
        if let Some((px, py, pr)) = previous {
            let dx: f64 = x - px;
            let dy: f64 = y - py;
            let len = dx.hypot(dy);
            if len > 0.0 {
                let nx = dy / len;
                let ny = -dx / len;
                emit(0, [px + nx * pr, py + ny * pr, 0.0, 0.0, 0.0, 0.0]);
                emit(1, [x + nx * r, y + ny * r, 0.0, 0.0, 0.0, 0.0]);
                emit(1, [x - nx * r, y - ny * r, 0.0, 0.0, 0.0, 0.0]);
                emit(1, [px - nx * pr, py - ny * pr, 0.0, 0.0, 0.0, 0.0]);
                emit(3, [0.0; 6]);
            }
        }
        // Clockwise circle and segment subpaths use matching winding. The native
        // painter fills their union once per stroke, retaining transparent brush alpha.
        let k = r * 0.552_284_749_830_793_6;
        emit(0, [x + r, y, 0.0, 0.0, 0.0, 0.0]);
        emit(2, [x + r, y + k, x + k, y + r, x, y + r]);
        emit(2, [x - k, y + r, x - r, y + k, x - r, y]);
        emit(2, [x - r, y - k, x - k, y - r, x, y - r]);
        emit(2, [x + k, y - r, x + r, y - k, x + r, y]);
        emit(3, [0.0; 6]);
        previous = Some((x, y, r));
    }
}
