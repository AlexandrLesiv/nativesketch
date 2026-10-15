use crate::{RenderError, RenderRequest, RenderStats, adornments, paths::Paths};
use sketch_core::{Bounds, RenderScene};
use skia_safe::{Color, Image, Surface};
#[derive(Clone, PartialEq)]
struct BackgroundKey {
    session: u64,
    revision: u64,
    viewport: [f64; 5],
    pixels: [u32; 2],
    context: u64,
}
struct Background {
    key: BackgroundKey,
    image: Image,
    bytes: usize,
}
/// One owner; no interior mutation shared between render threads.
pub struct Compositor {
    paths: Paths,
    background: Option<Background>,
    background_limit: usize,
    generation: u64,
    last: Option<RenderRequest>,
}
impl Default for Compositor {
    fn default() -> Self {
        Self::new(32 * 1024 * 1024, 64 * 1024 * 1024)
    }
}
impl Compositor {
    pub fn new(path_limit: usize, background_limit: usize) -> Self {
        Self {
            paths: Paths::new(path_limit),
            background: None,
            background_limit,
            generation: 1,
            last: None,
        }
    }
    pub fn reset(&mut self) -> Result<(), RenderError> {
        let next = self
            .generation
            .checked_add(1)
            .ok_or(RenderError::ResourceLimit)?;
        self.paths.clear();
        self.background = None;
        self.generation = next;
        Ok(())
    }
    pub fn validate(&self, request: &RenderRequest) -> Result<(), RenderError> {
        request.validate()?;
        if let Some(last) = self.last {
            if request.surface_generation < last.surface_generation
                || (request.session == last.session
                    && request.scene_generation < last.scene_generation)
            {
                return Err(RenderError::Stale);
            }
            if request.surface_generation == last.surface_generation
                && (request.pixel_width != last.pixel_width
                    || request.pixel_height != last.pixel_height
                    || request.logical_width != last.logical_width
                    || request.logical_height != last.logical_height)
            {
                return Err(RenderError::InvalidRequest);
            }
        }
        Ok(())
    }
    pub fn render(
        &mut self,
        surface: &mut Surface,
        scene: &RenderScene,
        request: &RenderRequest,
        selected: Option<Bounds>,
        cached: bool,
    ) -> Result<RenderStats, RenderError> {
        self.validate(request)?;
        if !request.matches(scene.key) {
            return Err(RenderError::Stale);
        }
        if surface.width() != request.pixel_width as i32
            || surface.height() != request.pixel_height as i32
        {
            return Err(RenderError::InvalidRequest);
        }
        let mut stats = RenderStats {
            context_generation: self.generation,
            ..Default::default()
        };
        let key = BackgroundKey {
            session: request.session,
            revision: request.revision,
            viewport: [
                request.offset_x,
                request.offset_y,
                request.zoom,
                request.logical_width,
                request.logical_height,
            ],
            pixels: [request.pixel_width, request.pixel_height],
            context: self.generation,
        };
        if self
            .last
            .is_some_and(|last| last.session != request.session)
        {
            self.paths.clear();
            self.background = None;
        }
        if scene.transform_preview.is_none() && cached {
            self.background(surface, scene, request, key, &mut stats)?;
        } else {
            self.draw_ordered(surface, scene, request, cached, &mut stats);
        }
        if let Some(live) = &scene.live {
            self.paths
                .draw(surface.canvas(), live, request, false, &mut stats);
        }
        adornments::draw(surface.canvas(), request, selected);
        stats.path_bytes = self.paths.bytes as u64;
        stats.background_bytes = self.background.as_ref().map_or(0, |b| b.bytes as u64);
        self.last = Some(*request);
        Ok(stats)
    }
    fn background(
        &mut self,
        surface: &mut Surface,
        scene: &RenderScene,
        request: &RenderRequest,
        key: BackgroundKey,
        stats: &mut RenderStats,
    ) -> Result<(), RenderError> {
        if self.background.as_ref().is_some_and(|b| b.key == key) {
            stats.background_hit = 1;
        } else {
            self.background = None;
            let bytes = request.pixel_width as usize * request.pixel_height as usize * 4;
            if bytes > self.background_limit {
                self.draw_ordered(surface, scene, request, true, stats);
                return Ok(());
            }
            let mut base = surface
                .new_surface(&surface.image_info())
                .ok_or(RenderError::Backend)?;
            self.draw_ordered(&mut base, scene, request, true, stats);
            self.background = Some(Background {
                key,
                image: base.image_snapshot(),
                bytes,
            });
        }
        surface.canvas().clear(Color::WHITE);
        if let Some(base) = &self.background {
            surface.canvas().draw_image(&base.image, (0, 0), None);
        }
        Ok(())
    }
    fn draw_ordered(
        &mut self,
        surface: &mut Surface,
        scene: &RenderScene,
        request: &RenderRequest,
        cached: bool,
        stats: &mut RenderStats,
    ) {
        let canvas = surface.canvas();
        canvas.clear(Color::WHITE);
        let preview = scene.transform_preview.as_ref();
        let mut replacement = preview.and_then(|p| p.replacement.as_ref());
        for stroke in scene.committed.iter() {
            if let Some(value) = replacement
                && value.order <= stroke.order
            {
                self.paths.draw(canvas, value, request, cached, stats);
                replacement = None;
            }
            if preview.is_some_and(|p| p.stroke_id == stroke.stroke_id) {
                continue;
            }
            self.paths.draw(canvas, stroke, request, cached, stats);
        }
        if let Some(value) = replacement {
            self.paths.draw(canvas, value, request, cached, stats);
        }
    }
}
