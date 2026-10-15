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
