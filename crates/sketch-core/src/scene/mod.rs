mod builder;
pub(crate) use builder::build;
#[derive(Debug, Clone, Copy)]
pub struct PathCommand {
    pub stroke_id: u64,
    pub rgba: [f64; 4],
    pub verb: u32,
    pub points: [f64; 6],
}
mod render_types;
pub use render_types::{
    RenderScene, RenderViewport, SceneCacheStats, SceneError, SceneLimits, SceneStroke,
    StrokeGeometry, TransformPreview,
};
mod cache;
mod geometry;
mod visibility;
pub use cache::SceneCache;
mod prepare;

mod viewport_budget;
mod viewport_clip;
mod viewport_contours;
mod viewport_preparation;
mod viewport_types;
pub use viewport_preparation::{prepare_viewport_path, viewport_preparation_needed};
pub use viewport_types::{PixelViewport, PreparationLimits, PreparedViewportPath};

#[cfg(test)]
mod viewport_tests;
