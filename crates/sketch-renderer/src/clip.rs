use crate::RenderError;
pub(crate) type Point = [f64; 2];
pub(crate) type Rectangle = [f64; 4];
pub(crate) const MAX_POINTS: usize = 262_144;
// Fallback only: maximum control-hull distance from an accepted chord in physical
// pixels. This geometric approximation bound is not an image-diff tolerance.
const CURVE_TOLERANCE_PIXELS: f64 = 1.0 / 1024.0;
pub(crate) fn outside(points: &[Point], rect: Rectangle) -> bool {
    (0..4).any(|side| points.iter().all(|point| !inside(*point, rect, side)))
}
fn inside(point: Point, rect: Rectangle, side: usize) -> bool {
    match side {
        0 => point[0] >= rect[0],
        1 => point[0] <= rect[2],
        2 => point[1] >= rect[1],
        _ => point[1] <= rect[3],
    }
}
pub(crate) fn push(points: &mut Vec<Point>, point: Point) -> Result<(), RenderError> {
    if points.len() == MAX_POINTS {
        return Err(RenderError::ResourceLimit);
    }
    points.push(point);
    Ok(())
}
fn intersection(a: Point, b: Point, rect: Rectangle, side: usize) -> Point {
    let axis = usize::from(side >= 2);
    let edge = rect[match side {
        0 => 0,
        1 => 2,
        2 => 1,
        _ => 3,
    }];
    let t = (edge - a[axis]) / (b[axis] - a[axis]);
    let other = 1 - axis;
    let coordinate = if t <= 0.5 {
        a[other] + (b[other] - a[other]) * t
    } else {
        b[other] + (a[other] - b[other]) * (1.0 - t)
    };
    let mut result = [0.0; 2];
    result[axis] = edge;
    result[other] = coordinate;
    result
}
/// Sutherland–Hodgman clipping in f64; contours retain winding and share one fill.
pub(crate) fn polygon(mut points: Vec<Point>, rect: Rectangle) -> Result<Vec<Point>, RenderError> {
    for side in 0..4 {
        let Some(mut previous) = points.last().copied() else {
            return Ok(points);
        };
        let mut output = Vec::new();
        for current in points {
            let was_inside = inside(previous, rect, side);
            let is_inside = inside(current, rect, side);
            if was_inside != is_inside {
                push(&mut output, intersection(previous, current, rect, side))?;
            }
            if is_inside {
                push(&mut output, current)?;
            }
            previous = current;
        }
        points = output;
    }
    Ok(points)
}
fn midpoint(a: Point, b: Point) -> Point {
    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]
}
fn flat(curve: [Point; 4]) -> bool {
    let [a, b, c, d] = curve;
    let delta = [d[0] - a[0], d[1] - a[1]];
    let length = delta[0].hypot(delta[1]);
    [b, c].into_iter().all(|p| {
        let distance = if length == 0.0 {
            (p[0] - a[0]).hypot(p[1] - a[1])
        } else {
            ((p[0] - a[0]) * delta[1] - (p[1] - a[1]) * delta[0]).abs() / length
        };
        distance <= CURVE_TOLERANCE_PIXELS
    })
}
/// Only unsafe giant curved contours use this bounded fallback. Curves outside a
/// common clipping half-plane can become chords without changing the visible fill.
pub(crate) fn curve(
    curve: [Point; 4],
    rect: Rectangle,
    depth: u32,
    points: &mut Vec<Point>,
) -> Result<(), RenderError> {
    if outside(&curve, rect) || flat(curve) {
        return push(points, curve[3]);
    }
    if depth == 32 {
        return Err(RenderError::ResourceLimit);
    }
    let [a, b, c, d] = curve;
    let ab = midpoint(a, b);
    let bc = midpoint(b, c);
    let cd = midpoint(c, d);
    let abc = midpoint(ab, bc);
    let bcd = midpoint(bc, cd);
    let center = midpoint(abc, bcd);
    self::curve([a, ab, abc, center], rect, depth + 1, points)?;
    self::curve([center, bcd, cd, d], rect, depth + 1, points)
}
