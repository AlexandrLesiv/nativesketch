//! Direct Ganesh/Metal painter: host owns drawable acquisition and presentation.
use crate::{
    native_fixture::{RcCommand, RcOutline},
    pen_replay::PcSnapshot,
};
use skia_safe::{self as sk, gpu as sg};
use std::{ffi::c_void, time::Instant};
#[repr(C)]
#[derive(Default)]
pub struct PcPaintMetrics {
    path_ms: f64,
    cache_build_ms: f64,
    paint_submit_ms: f64,
    cache_bytes: u64,
    cache_rebuilt: u32,
    reserved: u32,
}
type Paths = Vec<(sk::Path, sk::Paint)>;
struct Painter {
    context: sg::DirectContext,
    width: u32,
    height: u32,
    base: Paths,
    live: Paths,
    clip: Option<sk::Path>,
    image: Option<sk::Image>,
    base_revision: Option<u64>,
    live_revision: Option<u64>,
    cache_key: Option<(u64, u64)>,
    background: Option<sk::Image>,
    failed: bool,
}
fn matrix(a: [f64; 6]) -> sk::Matrix {
    sk::Matrix::new_all(
        a[0] as f32,
        a[2] as f32,
        a[4] as f32,
        a[1] as f32,
        a[3] as f32,
        a[5] as f32,
        0.,
        0.,
        1.,
    )
}
fn path(commands: &[RcCommand]) -> Result<sk::Path, String> {
    let mut p = sk::PathBuilder::new();
    for c in commands {
        let a = c.points;
        match c.verb {
            0 => {
                p.move_to((a[0] as f32, a[1] as f32));
            }
            1 => {
                p.line_to((a[0] as f32, a[1] as f32));
            }
            2 => {
                p.cubic_to(
                    (a[0] as f32, a[1] as f32),
                    (a[2] as f32, a[3] as f32),
                    (a[4] as f32, a[5] as f32),
                );
            }
            3 => {
                p.close();
            }
            _ => return Err("Invalid path verb".into()),
        }
    }
    Ok(p.detach())
}
unsafe fn borrowed<'a, T>(p: *const T, n: u64) -> Result<&'a [T], String> {
    let n = usize::try_from(n).map_err(|e| e.to_string())?;
    if n == 0 {
        return Ok(&[]);
    }
    if p.is_null() || n > isize::MAX as usize / std::mem::size_of::<T>() {
        return Err("Invalid snapshot array".into());
    }
    Ok(unsafe { std::slice::from_raw_parts(p, n) })
}
fn paths(commands: &[RcCommand], outlines: &[RcOutline]) -> Result<Paths, String> {
    outlines
        .iter()
        .map(|o| {
            let begin = usize::try_from(o.command_start).map_err(|e| e.to_string())?;
            let end = begin
                .checked_add(usize::try_from(o.command_count).map_err(|e| e.to_string())?)
                .ok_or("Path range overflow")?;
            let commands = commands.get(begin..end).ok_or("Invalid outline range")?;
            let mut paint = sk::Paint::default();
            paint.set_anti_alias(true);
            paint.set_style(sk::paint::Style::Fill);
            paint.set_color4f(
                sk::Color4f::new(
                    o.rgba[0] as f32,
                    o.rgba[1] as f32,
                    o.rgba[2] as f32,
                    o.rgba[3] as f32,
                ),
                None,
            );
            Ok((path(commands)?, paint))
        })
        .collect()
}
fn apply_view(canvas: &sk::Canvas, s: &PcSnapshot, clip: Option<&sk::Path>) {
    canvas.concat(&matrix(s.transform));
    if s.has_clip != 0 {
        canvas.clip_rect(sk::Rect::new(150., 130., 1450., 870.), None, true);
        if let Some(p) = clip {
            canvas.clip_path(p, None, true);
        }
    }
}
fn draw_paths(canvas: &sk::Canvas, paths: &Paths) {
    for (path, paint) in paths {
        canvas.draw_path(path, paint);
    }
}
fn draw_base(
    canvas: &sk::Canvas,
    s: &PcSnapshot,
    base: &Paths,
    clip: Option<&sk::Path>,
    image: Option<&sk::Image>,
) {
    canvas.clear(sk::Color::WHITE);
    canvas.save();
    apply_view(canvas, s, clip);
    if let Some(image) = image {
        canvas.save();
        canvas.concat(&matrix(s.image_transform));
        canvas.draw_image_with_sampling_options(
            image,
            (0., 0.),
            sk::SamplingOptions::new(sk::FilterMode::Linear, sk::MipmapMode::None),
            None,
        );
        canvas.restore();
    }
    draw_paths(canvas, base);
    canvas.restore();
}
impl Painter {
    fn reset(&mut self) {
        self.base.clear();
        self.live.clear();
        self.clip = None;
        self.image = None;
        self.background = None;
        self.base_revision = None;
        self.live_revision = None;
        self.cache_key = None;
    }
    unsafe fn render(
        &mut self,
        texture: *mut c_void,
        s: &PcSnapshot,
        policy: u32,
    ) -> Result<PcPaintMetrics, String> {
        if texture.is_null() || policy > 1 {
            return Err("Invalid texture or cache policy".into());
        }
        let mut metrics = PcPaintMetrics::default();
        let start = Instant::now();
        if self.base_revision != Some(s.document_revision) {
            self.base = paths(
                unsafe { borrowed(s.base_commands, s.base_command_count) }?,
                unsafe { borrowed(s.base_outlines, s.base_outline_count) }?,
            )?;
            self.base_revision = Some(s.document_revision);
        }
        if self.live_revision != Some(s.live_revision) {
            self.live = paths(
                unsafe { borrowed(s.live_commands, s.live_command_count) }?,
                unsafe { borrowed(s.live_outlines, s.live_outline_count) }?,
            )?;
            self.live_revision = Some(s.live_revision);
        }
        if s.has_clip != 0 && self.clip.is_none() {
            self.clip = Some(path(unsafe {
                borrowed(s.clip_commands, s.clip_command_count)
            }?)?);
        }
        if s.has_image != 0 && self.image.is_none() {
            if s.image_byte_count != 2048 * 2048 * 4 {
                return Err("Wrong fixture image size".into());
            }
            let bytes = unsafe { borrowed(s.image_rgba, s.image_byte_count) }?;
            let info = sk::ImageInfo::new(
                (2048, 2048),
                sk::ColorType::RGBA8888,
                sk::AlphaType::Unpremul,
                None,
            );
            self.image = Some(
                sk::images::raster_from_data(&info, sk::Data::new_copy(bytes), 2048 * 4)
                    .ok_or("Image decode storage failed")?,
            );
        }
        metrics.path_ms = start.elapsed().as_secs_f64() * 1000.;
        let cached = policy == 1 && s.cache_allowed != 0;
        if !cached {
            self.background = None;
            self.cache_key = None;
        }
        if cached && self.cache_key != Some((s.document_revision, s.viewport_revision)) {
            let start = Instant::now();
            let info = sk::ImageInfo::new(
                (self.width as i32, self.height as i32),
                sk::ColorType::BGRA8888,
                sk::AlphaType::Premul,
                None,
            );
            let mut replacement = sg::surfaces::render_target(
                &mut self.context,
                sg::Budgeted::Yes,
                &info,
                Some(0),
                sg::SurfaceOrigin::TopLeft,
                None,
                false,
                false,
            )
            .ok_or("Skia background allocation failed")?;
            draw_base(
                replacement.canvas(),
                s,
                &self.base,
                self.clip.as_ref(),
                self.image.as_ref(),
            );
            self.background = Some(replacement.image_snapshot());
            self.cache_key = Some((s.document_revision, s.viewport_revision));
            metrics.cache_build_ms = start.elapsed().as_secs_f64() * 1000.;
            metrics.cache_rebuilt = 1;
        }
        metrics.cache_bytes = if self.background.is_some() {
            self.width as u64 * self.height as u64 * 4
        } else {
            0
        };
        let start = Instant::now();
        let info = unsafe { sg::mtl::TextureInfo::new(texture as sg::mtl::Handle) };
        let target =
            sg::backend_render_targets::make_mtl((self.width as i32, self.height as i32), &info);
        let mut surface = sg::surfaces::wrap_backend_render_target(
            &mut self.context,
            &target,
            sg::SurfaceOrigin::TopLeft,
            sk::ColorType::BGRA8888,
            None,
            None,
        )
        .ok_or("Skia drawable wrap failed")?;
        let canvas = surface.canvas();
        if let Some(background) = self.background.as_ref().filter(|_| cached) {
            canvas.clear(sk::Color::WHITE);
            canvas.draw_image(background, (0., 0.), None);
        } else {
            draw_base(
                canvas,
                s,
                &self.base,
                self.clip.as_ref(),
                self.image.as_ref(),
            );
        }
        canvas.save();
        apply_view(canvas, s, self.clip.as_ref());
        draw_paths(canvas, &self.live);
        canvas.restore();
        self.context.flush_and_submit();
        metrics.paint_submit_ms = start.elapsed().as_secs_f64() * 1000.;
        Ok(metrics)
    }
}
fn call(h: *mut c_void, f: impl FnOnce(&mut Painter) -> Result<(), String>) -> i32 {
    let result = crate::ffi(|| {
        let p = unsafe { h.cast::<Painter>().as_mut() }.ok_or("Null Skia painter")?;
        if p.failed {
            return Err("Skia painter poisoned; destroy".into());
        }
        f(p)
    });
    if result != 0
        && let Some(p) = unsafe { h.cast::<Painter>().as_mut() }
    {
        p.failed = true;
    }
    result
}
/// # Safety
/// Native Metal device and queue live until destroy; see pen_comparison.h.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pc_skia_create(
    device: *mut c_void,
    queue: *mut c_void,
    width: u32,
    height: u32,
) -> *mut c_void {
    let mut result = std::ptr::null_mut();
    crate::ffi(|| {
        if device.is_null()
            || queue.is_null()
            || !(64..=8192).contains(&width)
            || !(64..=8192).contains(&height)
        {
            return Err("Invalid Skia create arguments".into());
        }
        let backend = unsafe {
            sg::mtl::BackendContext::new(device as sg::mtl::Handle, queue as sg::mtl::Handle)
        };
        let context = sg::direct_contexts::make_metal(&backend, None)
            .ok_or("Skia direct Metal context creation failed")?;
        result = Box::into_raw(Box::new(Painter {
            context,
            width,
            height,
            base: vec![],
            live: vec![],
            clip: None,
            image: None,
            base_revision: None,
            live_revision: None,
            cache_key: None,
            background: None,
            failed: false,
        }))
        .cast();
        Ok(())
    });
    result
}
/// # Safety
/// Snapshot arrays and texture valid for synchronous call; see pen_comparison.h.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pc_skia_render(
    h: *mut c_void,
    texture: *mut c_void,
    snapshot: *const PcSnapshot,
    policy: u32,
    out: *mut PcPaintMetrics,
) -> i32 {
    call(h, |p| {
        if snapshot.is_null() || out.is_null() {
            return Err("Null render snapshot/metrics".into());
        }
        unsafe { *out = p.render(texture, &*snapshot, policy)? };
        Ok(())
    })
}
/// # Safety
/// Handle serially owned and valid, see pen_comparison.h.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pc_skia_reset(h: *mut c_void) -> i32 {
    call(h, |p| {
        p.reset();
        Ok(())
    })
}
/// # Safety
/// Handle null or uniquely owned valid painter, destroyed once after host GPU retirement.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pc_skia_destroy(h: *mut c_void) {
    if !h.is_null() {
        crate::ffi(|| {
            drop(unsafe { Box::from_raw(h.cast::<Painter>()) });
            Ok(())
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_path_ranges_rejected() {
        assert!(
            paths(
                &[],
                &[RcOutline {
                    stroke_id: 1,
                    command_start: 0,
                    command_count: 1,
                    rgba: [1.; 4]
                }]
            )
            .is_err()
        );
        assert!(
            path(&[RcCommand {
                verb: 9,
                reserved: 0,
                points: [0.; 6]
            }])
            .is_err()
        );
    }
    #[test]
    fn no_gpu_null_arguments_fail() {
        unsafe {
            assert!(
                pc_skia_create(std::ptr::null_mut(), std::ptr::null_mut(), 1600, 1000).is_null()
            );
            assert_eq!(pc_skia_reset(std::ptr::null_mut()), 1);
        }
    }
}
