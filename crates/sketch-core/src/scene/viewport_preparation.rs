use super::{
    PixelViewport, PreparationLimits, PreparedViewportPath, SceneError, SceneStroke,
    viewport_budget::Budget, viewport_contours,
};
pub(super) const SAFE_EXTENT: f64 = 4096.0;
/// Conservative decision shared by native Skia and foreign renderers. No document mutation.
pub fn viewport_preparation_needed(
    stroke: &SceneStroke,
    v: PixelViewport,
) -> Result<bool, SceneError> {
    v.validate()?;
    let b = stroke.geometry.bounds;
    let t = stroke.transform;
    let scale = v.scale();
    let values = [
        (b.x * t.scale + t.tx - v.document.x) * scale[0],
        (b.y * t.scale + t.ty - v.document.y) * scale[1],
        b.width * t.scale * scale[0],
        b.height * t.scale * scale[1],
    ];
    if ![t.scale, t.tx, t.ty, b.x, b.y, b.width, b.height]
        .into_iter()
        .all(f64::is_finite)
        || t.scale <= 0.0
        || b.width < 0.0
        || b.height < 0.0
        || !values.into_iter().all(f64::is_finite)
    {
        return Err(SceneError::InvalidViewport);
    }
    Ok(values.into_iter().any(|v| v.abs() > SAFE_EXTENT))
}
/// Pixel-space output preserves whole contour winding and is disposable, never persisted.
/// CPU work is bounded but synchronous; call during frame preparation, not sample append.
pub fn prepare_viewport_path(
    stroke: &SceneStroke,
    v: PixelViewport,
    limits: PreparationLimits,
) -> Result<PreparedViewportPath, SceneError> {
    if !viewport_preparation_needed(stroke, v)? {
        return Ok(PreparedViewportPath::LocalGeometry);
    }
    let mut budget = Budget::new(limits)?;
    let commands = &stroke.geometry.commands;
    let mut output = Vec::new();
    let mut start = 0;
    for (i, command) in commands.iter().enumerate() {
        budget.tick()?;
        if command.verb == 0 && i != start {
            viewport_contours::emit(&commands[start..i], stroke, v, &mut output, &mut budget)?;
            start = i;
        }
    }
    if start < commands.len() {
        viewport_contours::emit(&commands[start..], stroke, v, &mut output, &mut budget)?;
    }
    Ok(PreparedViewportPath::PixelCommands(output))
}
