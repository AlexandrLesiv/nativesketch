use crate::*;
use sketch_core::{RenderScene, RenderViewport, SceneCache, SceneStroke, StrokeGeometry};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};
pub(crate) struct Lease {
    pub id: u64,
    pub session: SessionHandle,
    pub root: u64,
    pub viewport: RenderViewport,
    pub scene: RenderScene,
}
pub(crate) struct Geometry {
    pub id: u64,
    pub weak: Weak<StrokeGeometry>,
    pub pins: usize,
    pub bytes: usize,
}
pub(crate) struct Root {
    pub id: u64,
    pub weak: Weak<[SceneStroke]>,
    pub pins: usize,
    pub bytes: usize,
    pub geometries: Vec<usize>,
}
pub(crate) struct Scenes {
    pub limits: TransportLimits,
    pub leases: Vec<Lease>,
    pub geometries: HashMap<usize, Geometry>,
    pub roots: HashMap<usize, Root>,
    pub geometry_ids: HashMap<u64, usize>,
    pub cache: SceneCache,
    pub bytes: usize,
}
pub(crate) fn geometry_key(value: &Arc<StrokeGeometry>) -> usize {
    Arc::as_ptr(value) as usize
}
pub(crate) fn root_key(value: &Arc<[SceneStroke]>) -> usize {
    Arc::as_ptr(value).cast::<SceneStroke>() as usize
}
impl Scenes {
    pub fn new(limits: TransportLimits) -> Result<Self, WireError> {
        let mut leases = Vec::new();
        leases
            .try_reserve_exact(limits.scenes)
            .map_err(|_| WireError::capacity())?;
        let mut geometries = HashMap::new();
        geometries
            .try_reserve(limits.geometry_entries)
            .map_err(|_| WireError::capacity())?;
        let mut geometry_ids = HashMap::new();
        geometry_ids
            .try_reserve(limits.geometry_entries)
            .map_err(|_| WireError::capacity())?;
        let mut roots = HashMap::new();
        roots
            .try_reserve(limits.scenes + 1)
            .map_err(|_| WireError::capacity())?;
        Ok(Self {
            limits,
            leases,
            geometries,
            geometry_ids,
            roots,
            cache: SceneCache::default(),
            bytes: 0,
        })
    }
    pub fn lease(&self, id: u64) -> Result<&Lease, WireError> {
        self.leases
            .iter()
            .find(|s| s.id == id)
            .ok_or(WireError::plain(7))
    }
    pub fn geometry(&self, scene: u64, id: u64) -> Result<Arc<StrokeGeometry>, WireError> {
        let lease = self.lease(scene)?;
        let key = *self.geometry_ids.get(&id).ok_or(WireError::plain(7))?;
        let root = &self.roots[&root_key(&lease.scene.committed)];
        let in_root = root.geometries.binary_search(&key).is_ok();
        let in_overlay = lease
            .scene
            .live
            .iter()
            .chain(
                lease
                    .scene
                    .transform_preview
                    .iter()
                    .filter_map(|p| p.replacement.as_ref()),
            )
            .any(|s| geometry_key(&s.geometry) == key);
        if !in_root && !in_overlay {
            return Err(WireError::plain(7));
        }
        let geometry = self
            .geometries
            .get(&key)
            .and_then(|g| g.weak.upgrade())
            .ok_or(WireError::plain(7))?;
        Ok(geometry)
    }
    pub fn geometry_id(&self, stroke: &SceneStroke) -> u64 {
        self.geometries[&geometry_key(&stroke.geometry)].id
    }
    pub fn release(&mut self, id: u64) -> Result<(), WireError> {
        let index = self
            .leases
            .iter()
            .position(|s| s.id == id)
            .ok_or(WireError::plain(7))?;
        let lease = self.leases.swap_remove(index);
        let key = root_key(&lease.scene.committed);
        let root = self.roots.get_mut(&key).expect("leased root");
        root.pins -= 1;
        if root.pins == 0 {
            let root = self.roots.remove(&key).expect("leased root");
            self.bytes -= root.bytes;
            for key in root.geometries {
                self.unpin_geometry(key);
            }
        }
        for stroke in lease.scene.live.iter().chain(
            lease
                .scene
                .transform_preview
                .iter()
                .filter_map(|p| p.replacement.as_ref()),
        ) {
            self.unpin_geometry(geometry_key(&stroke.geometry));
        }
        Ok(())
    }
    fn unpin_geometry(&mut self, key: usize) {
        let geometry = self.geometries.get_mut(&key).expect("leased geometry");
        geometry.pins -= 1;
        if geometry.pins == 0 {
            let geometry = self.geometries.remove(&key).expect("leased geometry");
            self.geometry_ids.remove(&geometry.id);
            self.bytes -= geometry.bytes;
        }
    }
}
