use super::viewport_preparation::SAFE_EXTENT;
use super::{
    PathCommand, PixelViewport, SceneError, SceneStroke,
    viewport_budget::Budget,
    viewport_clip::{self, Point},
};
fn points(
    command: &PathCommand,
    stroke: &SceneStroke,
    v: PixelViewport,
) -> Result<([Point; 3], usize), SceneError> {
    let count = match command.verb {
        0 | 1 => 1,
        2 => 3,
        3 => 0,
        _ => return Err(SceneError::InvalidViewport),
    };
    let mut points = [[0.0; 2]; 3];
    let scale = v.scale();
    let t = stroke.transform;
    for (i, point) in points.iter_mut().enumerate().take(count) {
        point[0] = (command.points[i * 2] * t.scale + t.tx - v.document.x) * scale[0];
        point[1] = (command.points[i * 2 + 1] * t.scale + t.ty - v.document.y) * scale[1];
        if !point.iter().all(|v| v.is_finite()) {
            return Err(SceneError::InvalidViewport);
        }
    }
    Ok((points, count))
}
fn append(
    out: &mut Vec<PathCommand>,
    template: PathCommand,
    verb: u32,
    points: &[Point],
    budget: &mut Budget,
) -> Result<(), SceneError> {
    let mut command = PathCommand {
        verb,
        points: [0.0; 6],
        ..template
    };
    for (i, point) in points.iter().enumerate() {
        if !point.iter().all(|v| v.is_finite()) {
            return Err(SceneError::InvalidViewport);
        }
        budget.point()?;
        command.points[i * 2] = point[0];
        command.points[i * 2 + 1] = point[1];
    }
    budget.push(out, command)
}
pub(super) fn emit(
    commands: &[PathCommand],
    stroke: &SceneStroke,
    v: PixelViewport,
    out: &mut Vec<PathCommand>,
    budget: &mut Budget,
) -> Result<(), SceneError> {
    let rect = v.rectangle();
    let mut outside = [true; 4];
    let mut safe = true;
    for command in commands {
        budget.tick()?;
        let (points, count) = points(command, stroke, v)?;
        for point in points.iter().take(count) {
            for (side, value) in outside.iter_mut().enumerate() {
                *value &= !viewport_clip::inside(*point, rect, side);
            }
            safe &= point.iter().all(|value| value.abs() <= SAFE_EXTENT);
        }
    }
    if outside.into_iter().any(|v| v) {
        return Ok(());
    }
    if safe {
        for command in commands {
            budget.tick()?;
            let (points, count) = points(command, stroke, v)?;
            append(out, *command, command.verb, &points[..count], budget)?;
        }
        return Ok(());
    }
    let mut polygon = Vec::new();
    for command in commands {
        budget.tick()?;
        let (p, _) = points(command, stroke, v)?;
        match command.verb {
            0 | 1 => {
                if polygon.len() == budget.limits.points {
                    return Err(SceneError::ResourceLimit);
                }
                budget.push(&mut polygon, p[0])?;
            }
            2 => {
                let last = *polygon.last().ok_or(SceneError::InvalidViewport)?;
                viewport_clip::curve([last, p[0], p[1], p[2]], rect, 0, &mut polygon, budget)?;
            }
            _ => {}
        }
    }
    let polygon = viewport_clip::polygon(polygon, rect, budget)?;
    if let Some(point) = polygon.first() {
        let template = commands[0];
        append(out, template, 0, &[*point], budget)?;
        for point in &polygon[1..] {
            append(out, template, 1, &[*point], budget)?;
        }
        append(out, template, 3, &[], budget)?;
    }
    budget.release(polygon);
    Ok(())
}
