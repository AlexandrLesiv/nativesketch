use crate::{
    RenderError, RenderRequest,
    clip::{self, Point, Rectangle},
};
use sketch_core::{PathCommand, SceneStroke};
use skia_safe::{Path, PathBuilder};
const SAFE_EXTENT: f64 = 4096.0;
pub(crate) fn needed(stroke: &SceneStroke, request: &RenderRequest) -> bool {
    let bounds = stroke.geometry.bounds;
    let t = stroke.transform;
    let viewport = request.viewport();
    let density = request.scale();
    [
        (bounds.x * t.scale + t.tx - viewport.x) * request.zoom * density.0 as f64,
        (bounds.y * t.scale + t.ty - viewport.y) * request.zoom * density.1 as f64,
        bounds.width * t.scale * request.zoom * density.0 as f64,
        bounds.height * t.scale * request.zoom * density.1 as f64,
    ]
    .into_iter()
    .any(|v| v.abs() > SAFE_EXTENT)
}
fn points(command: &PathCommand, stroke: &SceneStroke, request: &RenderRequest) -> Vec<Point> {
    let count = match command.verb {
        0 | 1 => 1,
        2 => 3,
        _ => 0,
    };
    let t = stroke.transform;
    let v = request.viewport();
    let density = request.scale();
    (0..count)
        .map(|index| {
            [
                (command.points[index * 2] * t.scale + t.tx - v.x)
                    * request.zoom
                    * density.0 as f64,
                (command.points[index * 2 + 1] * t.scale + t.ty - v.y)
                    * request.zoom
                    * density.1 as f64,
            ]
        })
        .collect()
}
fn emit(
    builder: &mut PathBuilder,
    commands: &[(u32, Vec<Point>)],
    rect: Rectangle,
) -> Result<usize, RenderError> {
    let hull = commands
        .iter()
        .flat_map(|(_, points)| points.iter().copied())
        .collect::<Vec<_>>();
    if hull.is_empty() || clip::outside(&hull, rect) {
        return Ok(0);
    }
    if hull.iter().flatten().all(|v| v.abs() <= SAFE_EXTENT) {
        for (verb, points) in commands {
            let narrow = |index: usize| (points[index][0] as f32, points[index][1] as f32);
            match verb {
                0 => {
                    builder.move_to(narrow(0));
                }
                1 => {
                    builder.line_to(narrow(0));
                }
                2 => {
                    builder.cubic_to(narrow(0), narrow(1), narrow(2));
                }
                _ => {
                    builder.close();
                }
            }
        }
        return Ok(hull.len());
    }
    let mut polygon = Vec::new();
    for (verb, points) in commands {
        match verb {
            0 | 1 => clip::push(&mut polygon, points[0])?,
            2 => {
                let last = *polygon.last().ok_or(RenderError::InvalidRequest)?;
                clip::curve(
                    [last, points[0], points[1], points[2]],
                    rect,
                    0,
                    &mut polygon,
                )?;
            }
            _ => {}
        }
    }
    let polygon = clip::polygon(polygon, rect)?;
    if let Some(point) = polygon.first() {
        builder.move_to((point[0] as f32, point[1] as f32));
        for point in &polygon[1..] {
            builder.line_to((point[0] as f32, point[1] as f32));
        }
        builder.close();
    }
    Ok(polygon.len())
}
/// Rebase before narrowing. Clip enormous contours before Skia sees coordinates;
/// retain all subpaths in one winding fill to preserve translucent stroke unions.
pub(crate) fn build(stroke: &SceneStroke, request: &RenderRequest) -> Result<Path, RenderError> {
    let rect = [
        -2.0,
        -2.0,
        request.pixel_width as f64 + 2.0,
        request.pixel_height as f64 + 2.0,
    ];
    let mut builder = PathBuilder::new();
    let mut commands = Vec::new();
    let mut emitted = 0;
    for command in stroke.geometry.commands.iter() {
        if command.verb == 0 && !commands.is_empty() {
            emitted += emit(&mut builder, &commands, rect)?;
            if emitted > clip::MAX_POINTS {
                return Err(RenderError::ResourceLimit);
            }
            commands.clear();
        }
        if commands.len() == clip::MAX_POINTS {
            return Err(RenderError::ResourceLimit);
        }
        commands.push((command.verb, points(command, stroke, request)));
    }
    emitted += emit(&mut builder, &commands, rect)?;
    if emitted > clip::MAX_POINTS {
        return Err(RenderError::ResourceLimit);
    }
    Ok(builder.detach())
}
