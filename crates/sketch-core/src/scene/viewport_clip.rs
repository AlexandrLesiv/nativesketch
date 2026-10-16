use super::{SceneError, viewport_budget::Budget};
pub(super) type Point = [f64; 2];
pub(super) type Rectangle = [f64; 4];
const TOLERANCE: f64 = 1.0 / 1024.0;
pub(super) fn inside(point: Point, rect: Rectangle, side: usize) -> bool {
    match side {
        0 => point[0] >= rect[0],
        1 => point[0] <= rect[2],
        2 => point[1] >= rect[1],
        _ => point[1] <= rect[3],
    }
}
fn outside(points: &[Point], rect: Rectangle) -> bool {
    (0..4).any(|side| points.iter().all(|p| !inside(*p, rect, side)))
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
/// Sutherland–Hodgman in f64. Each contour retains winding; caller fills them together.
pub(super) fn polygon(
    mut points: Vec<Point>,
    rect: Rectangle,
    budget: &mut Budget,
) -> Result<Vec<Point>, SceneError> {
    for side in 0..4 {
        let Some(mut previous) = points.last().copied() else {
            return Ok(points);
        };
        let mut output = Vec::new();
        for &current in &points {
            budget.tick()?;
            let was = inside(previous, rect, side);
            let now = inside(current, rect, side);
            if was != now {
                budget.push(&mut output, intersection(previous, current, rect, side))?;
            }
            if now {
                budget.push(&mut output, current)?;
            }
            if output.len() > budget.limits.points {
                return Err(SceneError::ResourceLimit);
            }
            previous = current;
        }
        budget.release(points);
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
        distance <= TOLERANCE
    })
}
/// Only giant contours flatten. The control hull outside one half-plane can become a
/// chord without changing visible fill. Subdivision and intermediate capacity are bounded.
pub(super) fn curve(
    c: [Point; 4],
    rect: Rectangle,
    depth: u32,
    out: &mut Vec<Point>,
    budget: &mut Budget,
) -> Result<(), SceneError> {
    budget.tick()?;
    if outside(&c, rect) || flat(c) {
        if out.len() == budget.limits.points {
            return Err(SceneError::ResourceLimit);
        }
        return budget.push(out, c[3]);
    }
    if depth == budget.limits.depth {
        return Err(SceneError::ResourceLimit);
    }
    let [a, b, c, d] = c;
    let ab = midpoint(a, b);
    let bc = midpoint(b, c);
    let cd = midpoint(c, d);
    let abc = midpoint(ab, bc);
    let bcd = midpoint(bc, cd);
    let center = midpoint(abc, bcd);
    self::curve([a, ab, abc, center], rect, depth + 1, out, budget)?;
    self::curve([center, bcd, cd, d], rect, depth + 1, out, budget)
}
