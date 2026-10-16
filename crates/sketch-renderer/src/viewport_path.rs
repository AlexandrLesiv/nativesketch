use crate::{RenderError, RenderRequest};
use sketch_core::{
    PixelViewport, PreparationLimits, PreparedViewportPath, SceneError, SceneStroke,
};
use skia_safe::{Path, PathBuilder};
fn viewport(request: &RenderRequest) -> PixelViewport {
    PixelViewport {
        document: request.viewport(),
        pixel_width: request.pixel_width,
        pixel_height: request.pixel_height,
    }
}
fn error(error: SceneError) -> RenderError {
    match error {
        SceneError::ResourceLimit => RenderError::ResourceLimit,
        SceneError::InvalidViewport => RenderError::InvalidRequest,
    }
}
pub(crate) fn needed(stroke: &SceneStroke, request: &RenderRequest) -> Result<bool, RenderError> {
    sketch_core::viewport_preparation_needed(stroke, viewport(request)).map_err(error)
}
/// Convert shared f64 preparation only. Pixel commands already include transform and density.
pub(crate) fn build(stroke: &SceneStroke, request: &RenderRequest) -> Result<Path, RenderError> {
    let prepared =
        sketch_core::prepare_viewport_path(stroke, viewport(request), PreparationLimits::default())
            .map_err(error)?;
    let PreparedViewportPath::PixelCommands(commands) = prepared else {
        return Err(RenderError::InvalidRequest);
    };
    let mut builder = PathBuilder::new();
    for command in commands {
        let p = command.points;
        let point = |index: usize| (p[index * 2] as f32, p[index * 2 + 1] as f32);
        match command.verb {
            0 => {
                builder.move_to(point(0));
            }
            1 => {
                builder.line_to(point(0));
            }
            2 => {
                builder.cubic_to(point(0), point(1), point(2));
            }
            _ => {
                builder.close();
            }
        }
    }
    Ok(builder.detach())
}
