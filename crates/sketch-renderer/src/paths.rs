use crate::{RenderError, RenderStats, viewport_path};
use sketch_core::{SceneStroke, StrokeGeometry};
use skia_safe::{Canvas, Color4f, Paint, Path, PathBuilder};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};
struct Entry {
    _geometry: Weak<StrokeGeometry>,
    path: Path,
}
pub(crate) struct Paths {
    entries: HashMap<usize, Entry>,
    pub bytes: usize,
    limit: usize,
}
impl Paths {
    pub fn new(limit: usize) -> Self {
        Self {
            entries: HashMap::new(),
            bytes: 0,
            limit,
        }
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
    }
    fn path(&mut self, stroke: &SceneStroke, retain: bool, stats: &mut RenderStats) -> Path {
        let key = Arc::as_ptr(&stroke.geometry) as usize;
        if retain && let Some(entry) = self.entries.get(&key) {
            stats.path_hits += 1;
            return entry.path.clone();
        }
        stats.path_misses += 1;
        let mut builder = PathBuilder::new();
        let origin = stroke.geometry.origin;
        let point = |x: f64, y: f64| ((x - origin[0]) as f32, (y - origin[1]) as f32);
        for command in stroke.geometry.commands.iter() {
            let p = command.points;
            match command.verb {
                0 => {
                    builder.move_to(point(p[0], p[1]));
                }
                1 => {
                    builder.line_to(point(p[0], p[1]));
                }
                2 => {
                    builder.cubic_to(point(p[0], p[1]), point(p[2], p[3]), point(p[4], p[5]));
                }
                _ => {
                    builder.close();
                }
            }
        }
        let path = builder.detach();
        let bytes = path.approximate_bytes_used() + size_of::<Entry>() + size_of::<usize>();
        if retain && bytes <= self.limit {
            if bytes > self.limit - self.bytes {
                self.clear();
            }
            self.bytes += bytes;
            self.entries.insert(
                key,
                Entry {
                    _geometry: Arc::downgrade(&stroke.geometry),
                    path: path.clone(),
                },
            );
        }
        path
    }
    pub fn draw(
        &mut self,
        canvas: &Canvas,
        stroke: &SceneStroke,
        request: &crate::RenderRequest,
        retain: bool,
        stats: &mut RenderStats,
    ) -> Result<(), RenderError> {
        let Some(command) = stroke.geometry.commands.first() else {
            return Ok(());
        };
        let viewport_path = viewport_path::needed(stroke, request)?;
        let path = if viewport_path {
            stats.path_misses += 1;
            viewport_path::build(stroke, request)?
        } else {
            self.path(stroke, retain, stats)
        };
        let viewport = request.viewport();
        let origin = stroke.geometry.origin;
        let t = stroke.transform;
        let x = (origin[0] * t.scale + t.tx - viewport.x) * request.zoom;
        let y = (origin[1] * t.scale + t.ty - viewport.y) * request.zoom;
        let scale = (request.zoom * t.scale) as f32;
        let mut paint = Paint::new(
            Color4f::new(
                command.rgba[0] as f32,
                command.rgba[1] as f32,
                command.rgba[2] as f32,
                command.rgba[3] as f32,
            ),
            None,
        );
        paint.set_anti_alias(true);
        canvas.save();
        if !viewport_path {
            canvas.scale(request.scale());
            canvas.translate((x as f32, y as f32));
            canvas.scale((scale, scale));
        }
        canvas.draw_path(&path, &paint);
        canvas.restore();
        Ok(())
    }
}
