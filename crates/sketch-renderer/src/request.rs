use sketch_core::{RenderViewport, application::SceneKey};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    InvalidRequest,
    Stale,
    ResourceLimit,
    Backend,
}
/// Layout is mirrored by SketchRenderRequestV1 in the additive native ABI header.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct RenderRequest {
    pub struct_size: u32,
    pub focused: u32,
    pub session: u64,
    pub revision: u64,
    pub scene_generation: u64,
    pub surface_generation: u64,
    pub logical_width: f64,
    pub logical_height: f64,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub zoom: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub selection_rgba: [f64; 4],
    pub focus_rgba: [f64; 4],
}
impl RenderRequest {
    pub fn validate(&self) -> Result<(), RenderError> {
        let finite = [
            self.logical_width,
            self.logical_height,
            self.zoom,
            self.offset_x,
            self.offset_y,
        ]
        .into_iter()
        .all(f64::is_finite);
        let colors = self
            .selection_rgba
            .iter()
            .chain(&self.focus_rgba)
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v));
        if self.struct_size as usize != size_of::<Self>()
            || self.focused > 1
            || self.session == 0
            || self.surface_generation == 0
            || !finite
            || !colors
            || self.logical_width <= 0.0
            || self.logical_height <= 0.0
            || !(0.1..=16.0).contains(&self.zoom)
            || self.pixel_width == 0
            || self.pixel_height == 0
            || self.pixel_width > 16384
            || self.pixel_height > 16384
            || self.logical_width > 16384.0
            || self.logical_height > 16384.0
        {
            return Err(RenderError::InvalidRequest);
        }
        let densities = [
            self.pixel_width as f64 / self.logical_width,
            self.pixel_height as f64 / self.logical_height,
        ];
        if !densities
            .into_iter()
            .all(|v| v.is_finite() && (0.25..=16.0).contains(&v))
        {
            return Err(RenderError::InvalidRequest);
        }
        let viewport = self.viewport();
        if ![viewport.x, viewport.y, viewport.width, viewport.height]
            .into_iter()
            .all(f64::is_finite)
        {
            return Err(RenderError::InvalidRequest);
        }
        Ok(())
    }
    pub fn matches(&self, key: SceneKey) -> bool {
        self.session == key.session.get()
            && self.revision == key.revision.0
            && self.scene_generation == key.generation.get()
    }
    pub fn viewport(&self) -> RenderViewport {
        RenderViewport {
            x: -self.offset_x / self.zoom,
            y: -self.offset_y / self.zoom,
            width: self.logical_width / self.zoom,
            height: self.logical_height / self.zoom,
            margin: 2.0 / self.zoom,
        }
    }
    pub(crate) fn scale(&self) -> (f32, f32) {
        (
            (self.pixel_width as f64 / self.logical_width) as f32,
            (self.pixel_height as f64 / self.logical_height) as f32,
        )
    }
}
