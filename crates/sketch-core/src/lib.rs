//! Renderer-independent editable strokes and transactional editor state.
#![forbid(unsafe_code)]
pub mod application;
mod error;
mod geometry;
mod history;
mod interaction;
mod model;
mod persistence;
mod scene;
pub mod source_policy;
#[cfg(test)]
mod tests;
pub use application::{Editor, State};
pub use error::{Error, Result};
pub use model::{Bounds, Brush, Document, Input, Sample, Stroke, Transform};
pub use scene::PathCommand;
pub const MAX_SAMPLES: usize = 100_000;
pub const MAX_STROKES: usize = 10_000;
pub use scene::{
    RenderScene, RenderViewport, SceneCacheStats, SceneError, SceneLimits, SceneStroke,
    StrokeGeometry, TransformPreview,
};
