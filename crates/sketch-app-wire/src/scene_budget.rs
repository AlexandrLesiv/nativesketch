use crate::{scene_store::*, *};
use sketch_core::{PathCommand, RenderScene, RenderViewport, SceneStroke, StrokeGeometry};
use std::{collections::HashMap, mem::size_of, sync::Arc};
impl Scenes {
    pub fn publish(
        &mut self,
        session: SessionHandle,
        viewport: RenderViewport,
        scene: RenderScene,
        fresh: u64,
    ) -> Result<u64, WireError> {
        let root_key = root_key(&scene.committed);
        let known_root = self.roots.get(&root_key);
        let mut root_geometries = Vec::new();
        let mut pending: HashMap<usize, (&Arc<StrokeGeometry>, usize)> = HashMap::new();
        let count = if known_root.is_some() {
            2
        } else {
            scene.committed.len() + 2
        };
        pending
            .try_reserve(count)
            .map_err(|_| WireError::capacity())?;
        root_geometries
            .try_reserve_exact(if known_root.is_some() {
                0
            } else {
                scene.committed.len()
            })
            .map_err(|_| WireError::capacity())?;
        let committed = if known_root.is_some() {
            &[][..]
        } else {
            scene.committed.as_ref()
        };
        for stroke in committed {
            let key = geometry_key(&stroke.geometry);
            root_geometries.push(key);
            pending.entry(key).or_insert((&stroke.geometry, 0)).1 += 1;
        }
        for stroke in scene.live.iter().chain(
            scene
                .transform_preview
                .iter()
                .filter_map(|p| p.replacement.as_ref()),
        ) {
            pending
                .entry(geometry_key(&stroke.geometry))
                .or_insert((&stroke.geometry, 0))
                .1 += 1;
        }
        let root_bytes = if known_root.is_some() {
            0
        } else {
            scene
                .committed
                .len()
                .checked_mul(size_of::<SceneStroke>() + size_of::<usize>())
                .and_then(|n| n.checked_add(64))
                .ok_or(WireError::capacity())?
        };
        let mut additional = root_bytes;
        let mut new_count = 0;
        for (key, (geometry, _)) in &pending {
            if !self.geometries.contains_key(key) {
                new_count += 1;
                additional = additional
                    .checked_add(geometry_bytes(geometry)?)
                    .ok_or(WireError::capacity())?;
            }
        }
        if additional > self.limits.scene_bytes.saturating_sub(self.bytes)
            || new_count
                > self
                    .limits
                    .geometry_entries
                    .saturating_sub(self.geometries.len())
        {
            return Err(WireError::capacity());
        }
        root_geometries.sort_unstable();
        let mut next = fresh + 1;
        for (key, (geometry, pins)) in pending {
            if let Some(existing) = self.geometries.get_mut(&key) {
                existing.pins += pins;
            } else {
                let bytes = geometry_bytes(geometry).expect("preflighted geometry");
                self.geometries.insert(
                    key,
                    Geometry {
                        id: next,
                        weak: Arc::downgrade(geometry),
                        pins,
                        bytes,
                    },
                );
                self.geometry_ids.insert(next, key);
                next += 1;
            }
        }
        let root = if let Some(root) = self.roots.get_mut(&root_key) {
            assert!(root.weak.ptr_eq(&Arc::downgrade(&scene.committed)));
            root.pins += 1;
            root.id
        } else {
            self.roots.insert(
                root_key,
                Root {
                    id: next,
                    weak: Arc::downgrade(&scene.committed),
                    pins: 1,
                    bytes: root_bytes,
                    geometries: root_geometries,
                },
            );
            next
        };
        self.leases.push(Lease {
            id: fresh,
            session,
            root,
            viewport,
            scene,
        });
        self.bytes += additional;
        Ok(fresh)
    }
}
fn geometry_bytes(geometry: &StrokeGeometry) -> Result<usize, WireError> {
    geometry
        .commands
        .len()
        .checked_mul(size_of::<PathCommand>())
        .and_then(|n| n.checked_add(size_of::<StrokeGeometry>() + 64))
        .ok_or(WireError::capacity())
}
