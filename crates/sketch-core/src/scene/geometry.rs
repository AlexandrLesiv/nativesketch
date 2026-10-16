use super::{SceneError, StrokeGeometry};
use crate::{Bounds, PathCommand, Sample, Stroke, Transform};
use std::{mem::size_of, sync::Arc};

/// Payload plus conservative Arc and cache-entry bookkeeping. Allocator overhead is excluded.
pub(super) fn geometry_bytes(commands: usize) -> Result<usize, SceneError> {
    commands
        .checked_mul(size_of::<PathCommand>())
        .and_then(|n| n.checked_add(size_of::<StrokeGeometry>() + 64))
        .ok_or(SceneError::ResourceLimit)
}
pub(super) fn sample_bounds(stroke: &Stroke, samples: impl Iterator<Item = Sample>) -> Bounds {
    let mut low = [f64::INFINITY; 2];
    let mut high = [f64::NEG_INFINITY; 2];
    for sample in samples {
        let radius = crate::geometry::radius(stroke, sample);
        low[0] = low[0].min(sample.x - radius);
        low[1] = low[1].min(sample.y - radius);
        high[0] = high[0].max(sample.x + radius);
        high[1] = high[1].max(sample.y + radius);
    }
    Bounds {
        x: low[0],
        y: low[1],
        width: high[0] - low[0],
        height: high[1] - low[1],
    }
}
pub(super) fn build_geometry(
    stroke: &Stroke,
    samples: impl Iterator<Item = Sample> + Clone,
    bounds: Bounds,
    maximum: usize,
) -> Result<Arc<StrokeGeometry>, SceneError> {
    let mut count = 0usize;
    let mut previous = None;
    for sample in samples.clone() {
        count = count.checked_add(6).ok_or(SceneError::ResourceLimit)?;
        let point = (sample.x, sample.y);
        if previous.is_some_and(|p| p != point) {
            count = count.checked_add(5).ok_or(SceneError::ResourceLimit)?;
        }
        previous = Some(point);
    }
    if geometry_bytes(count)? > maximum {
        return Err(SceneError::ResourceLimit);
    }
    let mut commands = Vec::new();
    commands
        .try_reserve_exact(count)
        .map_err(|_| SceneError::ResourceLimit)?;
    crate::geometry::render_stroke(stroke, Transform::default(), samples, &mut commands);
    Ok(Arc::new(StrokeGeometry {
        commands: commands.into(),
        bounds,
        origin: [bounds.x, bounds.y],
    }))
}
