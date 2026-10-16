use super::{cache::CommittedRoot, geometry::*, *};
use crate::{Document, Stroke, application::SceneKey, interaction::ActiveInteraction};
use std::mem::size_of;

impl SceneCache {
    pub(crate) fn build(
        &mut self,
        document: &Document,
        interaction: &ActiveInteraction,
        key: SceneKey,
        viewport: RenderViewport,
    ) -> Result<RenderScene, SceneError> {
        viewport.validate()?;
        let reuse = self.session == Some(key.session)
            && self
                .root
                .as_ref()
                .is_some_and(|root| root.revision == key.revision && root.viewport == viewport);
        if reuse {
            self.stats.committed_reuses = self.stats.committed_reuses.saturating_add(1);
        } else {
            self.prepare(key.session, document);
            let mut strokes = Vec::new();
            let mut bytes = 2 * size_of::<usize>();
            if bytes > self.limits.prepared_bytes {
                return Err(SceneError::ResourceLimit);
            }
            for (order, stroke) in document.strokes.iter().enumerate() {
                let bounds = self.bounds(stroke);
                if viewport.intersects(bounds, stroke.transform) {
                    let geometry = self.geometry(stroke, bounds, self.remaining(bytes)?)?;
                    bytes += geometry_bytes(geometry.commands.len())? + size_of::<SceneStroke>();
                    strokes.push(SceneStroke {
                        stroke_id: stroke.id,
                        order,
                        transform: stroke.transform,
                        geometry,
                    });
                }
            }
            self.root = Some(CommittedRoot {
                revision: key.revision,
                viewport,
                strokes: strokes.into(),
                bytes,
            });
            self.stats.committed_builds = self.stats.committed_builds.saturating_add(1);
        }
        let root = self.root.as_ref().expect("prepared root");
        let committed = root.strokes.clone();
        let bytes = root.bytes;
        let mut scene = RenderScene {
            key,
            committed,
            live: None,
            transform_preview: None,
        };
        match interaction {
            ActiveInteraction::Drawing(ink) => {
                let samples = ink.stroke.samples.iter().chain(&ink.predicted).copied();
                if samples.clone().next().is_some() {
                    let bounds = sample_bounds(&ink.stroke, samples.clone());
                    if viewport.intersects(bounds, ink.stroke.transform) {
                        let geometry =
                            build_geometry(&ink.stroke, samples, bounds, self.remaining(bytes)?)?;
                        self.stats.geometry_builds = self.stats.geometry_builds.saturating_add(1);
                        scene.live = Some(SceneStroke {
                            stroke_id: ink.stroke.id,
                            order: document.strokes.len(),
                            transform: ink.stroke.transform,
                            geometry,
                        });
                    }
                }
            }
            ActiveInteraction::Transforming(preview) => {
                let stroke = &document.strokes[preview.index];
                let bounds = self.bounds(stroke);
                let replacement = if viewport.intersects(bounds, preview.preview) {
                    Some(self.preview_stroke(stroke, preview.index, preview.preview, bytes)?)
                } else {
                    None
                };
                scene.transform_preview = Some(TransformPreview {
                    stroke_id: stroke.id,
                    order: preview.index,
                    replacement,
                });
            }
            ActiveInteraction::Idle => {}
        }
        Ok(scene)
    }
    fn remaining(&self, bytes: usize) -> Result<usize, SceneError> {
        self.limits
            .prepared_bytes
            .checked_sub(bytes)
            .and_then(|n| n.checked_sub(size_of::<SceneStroke>()))
            .ok_or(SceneError::ResourceLimit)
    }
    fn preview_stroke(
        &mut self,
        stroke: &Stroke,
        order: usize,
        transform: crate::Transform,
        bytes: usize,
    ) -> Result<SceneStroke, SceneError> {
        // The original root already accounts for shared geometry; only the preview descriptor is new.
        let existing = self.root.as_ref().and_then(|root| {
            root.strokes
                .iter()
                .find(|item| item.stroke_id == stroke.id)
                .map(|item| item.geometry.clone())
        });
        let maximum = self.remaining(bytes)?;
        let geometry = if let Some(geometry) = existing {
            geometry
        } else {
            let bounds = self.bounds(stroke);
            self.geometry(stroke, bounds, maximum)?
        };
        Ok(SceneStroke {
            stroke_id: stroke.id,
            order,
            transform,
            geometry,
        })
    }
}
