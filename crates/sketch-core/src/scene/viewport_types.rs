use super::{PathCommand, RenderViewport, SceneError};

/// Actual physical backing dimensions mapped to the leased document viewport.
#[derive(Debug, Clone, Copy)]
pub struct PixelViewport {
    pub document: RenderViewport,
    pub pixel_width: u32,
    pub pixel_height: u32,
}
impl PixelViewport {
    pub(crate) fn validate(self) -> Result<(), SceneError> {
        let v = self.document;
        if ![v.x, v.y, v.width, v.height, v.margin]
            .into_iter()
            .all(f64::is_finite)
            || v.width <= 0.0
            || v.height <= 0.0
            || v.margin < 0.0
            || self.pixel_width == 0
            || self.pixel_height == 0
            || self.pixel_width > 16384
            || self.pixel_height > 16384
            || !self.scale().into_iter().all(|v| v.is_finite() && v > 0.0)
        {
            return Err(SceneError::InvalidViewport);
        }
        Ok(())
    }
    pub(crate) fn scale(self) -> [f64; 2] {
        [
            self.pixel_width as f64 / self.document.width,
            self.pixel_height as f64 / self.document.height,
        ]
    }
    pub(crate) fn rectangle(self) -> [f64; 4] {
        [
            -2.0,
            -2.0,
            self.pixel_width as f64 + 2.0,
            self.pixel_height as f64 + 2.0,
        ]
    }
}
/// Aggregate scratch/output capacity, point count and subdivision work are independent caps.
#[derive(Debug, Clone, Copy)]
pub struct PreparationLimits {
    pub bytes: usize,
    pub points: usize,
    pub work: usize,
    pub depth: u32,
}
impl Default for PreparationLimits {
    fn default() -> Self {
        Self {
            bytes: 64 * 1024 * 1024,
            points: 262_144,
            work: 8_388_608,
            depth: 32,
        }
    }
}
#[derive(Debug)]
pub enum PreparedViewportPath {
    /// Existing origin-localized immutable geometry can be used without viewport clipping.
    LocalGeometry,
    /// Whole stroke in physical pixel coordinates; fill once without another transform.
    PixelCommands(Vec<PathCommand>),
}
impl PreparedViewportPath {
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + match self {
                Self::LocalGeometry => 0,
                Self::PixelCommands(v) => v.capacity() * size_of::<PathCommand>(),
            }
    }
}
