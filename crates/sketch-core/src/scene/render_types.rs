use crate::application::SceneKey;
use crate::{Bounds, PathCommand, Transform};
use std::sync::Arc;

/// Document-space visible rectangle and conservative antialiasing expansion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderViewport {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub margin: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneLimits {
    pub retained_bytes: usize,
    pub prepared_bytes: usize,
}
impl Default for SceneLimits {
    fn default() -> Self {
        Self {
            retained_bytes: 32 * 1024 * 1024,
            prepared_bytes: 64 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneError {
    InvalidViewport,
    ResourceLimit,
}

/// Immutable local-space outline. Renderer narrowing must first subtract `origin`.
#[derive(Debug)]
pub struct StrokeGeometry {
    pub commands: Arc<[PathCommand]>,
    pub bounds: Bounds,
    pub origin: [f64; 2],
}

#[derive(Debug, Clone)]
pub struct SceneStroke {
    pub stroke_id: u64,
    /// Original document position: transform previews must retain this order.
    pub order: usize,
    pub transform: Transform,
    pub geometry: Arc<StrokeGeometry>,
}

#[derive(Debug, Clone)]
pub struct TransformPreview {
    /// Omit this original stroke from committed presentation, even if preview is culled.
    pub stroke_id: u64,
    pub order: usize,
    pub replacement: Option<SceneStroke>,
}

#[derive(Debug, Clone)]
pub struct RenderScene {
    pub key: SceneKey,
    /// Original committed transforms, shared unchanged throughout live input frames.
    pub committed: Arc<[SceneStroke]>,
    /// Actual and predicted input form ONE path/fill; never paint them independently.
    pub live: Option<SceneStroke>,
    pub transform_preview: Option<TransformPreview>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SceneCacheStats {
    pub retained_bytes: usize,
    pub retained_entries: usize,
    pub geometry_builds: u64,
    pub committed_builds: u64,
    pub committed_reuses: u64,
}
