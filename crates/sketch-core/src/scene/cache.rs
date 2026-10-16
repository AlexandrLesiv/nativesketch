use super::{geometry::*, *};
use crate::application::{DocumentSessionId, Revision};
use crate::{Bounds, Document, Stroke};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

pub(super) struct CommittedRoot {
    pub revision: Revision,
    pub viewport: RenderViewport,
    pub strokes: Arc<[SceneStroke]>,
    pub bytes: usize,
}
/// Disposable, single-owner preparation cache. No document mutation or renderer resources.
/// Limits account payload/bookkeeping, not allocator overhead. One retained root is separately
/// bounded by prepared_bytes; consumers must bound outstanding immutable snapshots as well.
pub struct SceneCache {
    pub(super) limits: SceneLimits,
    pub(super) session: Option<DocumentSessionId>,
    pub(super) root: Option<CommittedRoot>,
    bounds: HashMap<u64, Bounds>,
    entries: HashMap<u64, Arc<StrokeGeometry>>,
    insertion_order: VecDeque<u64>,
    pub(super) stats: SceneCacheStats,
}
impl Default for SceneCache {
    fn default() -> Self {
        Self::new(SceneLimits::default())
    }
}
impl SceneCache {
    pub fn new(limits: SceneLimits) -> Self {
        Self {
            limits,
            session: None,
            root: None,
            bounds: HashMap::new(),
            entries: HashMap::new(),
            insertion_order: VecDeque::new(),
            stats: SceneCacheStats::default(),
        }
    }
    /// Drops retained roots/geometry; already issued snapshots remain valid.
    pub fn clear(&mut self) {
        self.session = None;
        self.root = None;
        self.bounds.clear();
        self.entries.clear();
        self.insertion_order.clear();
        self.stats.retained_bytes = 0;
        self.stats.retained_entries = 0;
    }
    pub fn stats(&self) -> SceneCacheStats {
        self.stats
    }
    pub(super) fn prepare(&mut self, session: DocumentSessionId, document: &Document) {
        if self.session != Some(session) {
            self.clear();
            self.session = Some(session);
        }
        let ids: HashSet<_> = document.strokes.iter().map(|stroke| stroke.id).collect();
        self.bounds.retain(|id, _| ids.contains(id));
    }
    pub(super) fn bounds(&mut self, stroke: &Stroke) -> Bounds {
        *self
            .bounds
            .entry(stroke.id)
            .or_insert_with(|| sample_bounds(stroke, stroke.samples.iter().copied()))
    }
    pub(super) fn geometry(
        &mut self,
        stroke: &Stroke,
        bounds: Bounds,
        maximum: usize,
    ) -> Result<Arc<StrokeGeometry>, SceneError> {
        if let Some(geometry) = self.entries.get(&stroke.id) {
            if geometry_bytes(geometry.commands.len())? > maximum {
                return Err(SceneError::ResourceLimit);
            }
            return Ok(geometry.clone());
        }
        let geometry = build_geometry(stroke, stroke.samples.iter().copied(), bounds, maximum)?;
        self.stats.geometry_builds = self.stats.geometry_builds.saturating_add(1);
        let bytes = geometry_bytes(geometry.commands.len())?;
        if bytes <= self.limits.retained_bytes {
            while self.stats.retained_bytes > self.limits.retained_bytes - bytes {
                let id = self
                    .insertion_order
                    .pop_front()
                    .expect("retained byte accounting");
                let old = self.entries.remove(&id).expect("retained entry accounting");
                self.stats.retained_bytes -= geometry_bytes(old.commands.len())?;
            }
            self.entries.insert(stroke.id, geometry.clone());
            self.insertion_order.push_back(stroke.id);
            self.stats.retained_bytes += bytes;
            self.stats.retained_entries = self.entries.len();
        }
        Ok(geometry)
    }
}
