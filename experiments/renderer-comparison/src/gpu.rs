//! Experiment-only shared Metal device, queue, offscreen texture and presenter.
//! All operations are serialized by the host. Imported objects are retained and never destroyed
//! through HAL handles. Backend work and presentation use the SAME tracked-resource queue.
use crate::fixtures::{Fixture, Outline, clip_curve, image_transform, transform};
use objc2::{Message, rc::Retained, runtime::ProtocolObject};
use objc2_metal::*;
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};
use skia_safe::{self as sk, gpu as sg};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};
use vello_common::{
    kurbo::{Affine, BezPath, Rect},
    paint::{Image, ImageSource},
    peniko::{
        Fill, ImageQuality,
        color::{AlphaColor, Srgb},
    },
    pixmap::{PixelMetadata, Pixmap},
};

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub metal_device: Retained<ProtocolObject<dyn MTLDevice>>,
    pub metal_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub raw_texture: Retained<ProtocolObject<dyn MTLTexture>>,
    pub layer: Retained<CAMetalLayer>,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub timing_enabled: bool,
    marker: Retained<ProtocolObject<dyn MTLBuffer>>,
}
pub struct Ticket {
    pub begin: Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>,
    pub end: Retained<ProtocolObject<dyn MTLCommandBuffer>>,
    pub presented: Arc<AtomicU64>,
}
fn parse_gpu_timing(value: Option<&str>) -> Result<bool, String> {
    match value {
        None | Some("on") => Ok(true),
        Some("off") => Ok(false),
        Some(_) => Err("RENDERER_GPU_TIMING must be exactly on or off".into()),
    }
}
impl Gpu {
    /// Layer is an AppKit-owned CAMetalLayer, borrowed only for this call; retain extends lifetime.
    pub unsafe fn new(
        layer: *mut CAMetalLayer,
        width: u32,
        height: u32,
        paced: bool,
    ) -> Result<Self, String> {
        let timing_setting = match std::env::var("RENDERER_GPU_TIMING") {
            Ok(value) => Some(value),
            Err(std::env::VarError::NotPresent) => None,
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err("RENDERER_GPU_TIMING must be UTF-8 on or off".into());
            }
        };
        let timing_enabled = parse_gpu_timing(timing_setting.as_deref())?;
        let layer = unsafe { Retained::retain(layer) }.ok_or("Null Metal layer")?;
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            force_fallback_adapter: false,
            ..Default::default()
        }))
        .map_err(|e| e.to_string())?;
        let info = adapter.get_info();
        if info.backend != wgpu::Backend::Metal || info.device_type == wgpu::DeviceType::Cpu {
            return Err("Real Metal GPU required".into());
        }
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Renderer comparison shared device"),
            ..Default::default()
        }))
        .map_err(|e| e.to_string())?;
        let metal_device = unsafe { device.as_hal::<wgpu::hal::api::Metal>() }
            .ok_or("Metal device unavailable")?
            .raw_device()
            .clone();
        let metal_queue = unsafe { queue.as_hal::<wgpu::hal::api::Metal>() }
            .ok_or("Metal queue unavailable")?
            .as_raw()
            .retain();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Common BGRA8 output"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        // Seed wgpu's initialization tracker BEFORE Skia writes through the raw Metal handle.
        // Otherwise a later wgpu readback could clear an output it believes is uninitialized.
        // Both candidates execute this same untimed initialization and completion wait.
        let mut initialize = device.create_command_encoder(&Default::default());
        {
            let _pass = initialize.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Common target initialization"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }
        queue.submit([initialize.finish()]);
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| e.to_string())?;
        let raw_texture = unsafe { texture.as_hal::<wgpu::hal::api::Metal>() }
            .ok_or("Metal texture unavailable")?
            .raw_handle()
            .retain();
        // Runtime gate: external Skia/presenter accesses rely on Metal hazard tracking.
        if raw_texture.hazardTrackingMode() == MTLHazardTrackingMode::Untracked {
            return Err("Shared output texture must use tracked Metal hazards".into());
        }
        layer.setDevice(Some(&metal_device));
        layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        layer.setFramebufferOnly(false);
        layer.setPresentsWithTransaction(false);
        layer.setDisplaySyncEnabled(paced);
        layer.setMaximumDrawableCount(2);
        layer.setAllowsNextDrawableTimeout(true);
        let marker = metal_device
            .newBufferWithLength_options(4, MTLResourceOptions::StorageModeShared)
            .ok_or("Marker allocation failed")?;
        Ok(Self {
            device,
            queue,
            metal_device,
            metal_queue,
            texture,
            view,
            raw_texture,
            layer,
            name: info.name,
            width,
            height,
            timing_enabled,
            marker,
        })
    }
    pub fn acquire(&self) -> Result<Retained<ProtocolObject<dyn CAMetalDrawable>>, String> {
        let d = self
            .layer
            .nextDrawable()
            .ok_or("Drawable unavailable; trial invalidated")?;
        if d.texture().width() != self.width as usize
            || d.texture().height() != self.height as usize
        {
            return Err("Drawable size changed; trial invalidated".into());
        }
        Ok(d)
    }
    pub fn begin(&self) -> Result<Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>, String> {
        // Calibration changes only the profiling marker; ordinary rendering, final blit,
        // presentation and retirement remain identical.
        if !self.timing_enabled {
            return Ok(None);
        }
        let b = self
            .metal_queue
            .commandBuffer()
            .ok_or("Command buffer unavailable")?;
        let enc = b.blitCommandEncoder().ok_or("Marker encoder unavailable")?;
        enc.fillBuffer_range_value(&self.marker, objc2_foundation::NSRange::new(0, 4), 0);
        enc.endEncoding();
        b.commit();
        Ok(Some(b))
    }
    pub fn present(
        &self,
        begin: Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>,
        drawable: Retained<ProtocolObject<dyn CAMetalDrawable>>,
    ) -> Result<Ticket, String> {
        let end = self
            .metal_queue
            .commandBuffer()
            .ok_or("Present command buffer unavailable")?;
        let enc = end
            .blitCommandEncoder()
            .ok_or("Present encoder unavailable")?;
        unsafe {
            enc.copyFromTexture_sourceSlice_sourceLevel_sourceOrigin_sourceSize_toTexture_destinationSlice_destinationLevel_destinationOrigin(&self.raw_texture,0,0,MTLOrigin{x:0,y:0,z:0},MTLSize{width:self.width as usize,height:self.height as usize,depth:1},&drawable.texture(),0,0,MTLOrigin{x:0,y:0,z:0})
        };
        enc.endEncoding();
        let presented = Arc::new(AtomicU64::new(0));
        let p = presented.clone();
        let callback = block2::RcBlock::new(
            move |d: std::ptr::NonNull<ProtocolObject<dyn MTLDrawable>>| {
                let t = unsafe { d.as_ref() }.presentedTime();
                if t > 0.0 {
                    p.store(t.to_bits(), Ordering::Release);
                }
            },
        );
        unsafe { drawable.addPresentedHandler(block2::RcBlock::as_ptr(&callback)) };
        let base: Retained<ProtocolObject<dyn MTLDrawable>> = (&drawable).into();
        end.presentDrawable(&base);
        end.commit();
        Ok(Ticket {
            begin,
            end,
            presented,
        })
    }
    pub fn readback(&self) -> Result<Vec<u8>, String> {
        let stride = (self.width * 4).next_multiple_of(256);
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Untimed readback"),
            size: stride as u64 * self.height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r.map_err(|e| e.to_string()));
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| e.to_string())?;
        rx.recv().map_err(|e| e.to_string())??;
        let mapped = buffer
            .slice(..)
            .get_mapped_range()
            .map_err(|e| e.to_string())?;
        let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
        for row in mapped.chunks_exact(stride as usize) {
            for p in row[..self.width as usize * 4].as_chunks::<4>().0.iter() {
                rgba.extend_from_slice(&[p[2], p[1], p[0], p[3]]);
            }
        }
        drop(mapped);
        buffer.unmap();
        Ok(rgba)
    }
}
pub enum Backend {
    Skia(Skia),
    Vello(Box<Vello>),
}
pub struct Skia {
    context: sg::DirectContext,
    surface: sk::Surface,
    paths: Vec<(sk::Path, sk::Paint)>,
    image: Option<sk::Image>,
}
pub struct Vello {
    renderer: vello_gpu::Renderer,
    resources: vello_gpu::Resources,
    scene: vello_gpu::Scene,
    paths: Vec<(BezPath, AlphaColor<Srgb>)>,
    depth: wgpu::TextureView,
    image: Option<Image>,
    pending: Option<wgpu::CommandEncoder>,
}
impl Backend {
    pub fn new(name: &str, g: &Gpu) -> Result<Self, String> {
        match name {
            "skia" => {
                let backend = unsafe {
                    sg::mtl::BackendContext::new(
                        Retained::as_ptr(&g.metal_device) as sg::mtl::Handle,
                        Retained::as_ptr(&g.metal_queue) as sg::mtl::Handle,
                    )
                };
                let mut context = sg::direct_contexts::make_metal(&backend, None)
                    .ok_or("Skia Metal initialization failed")?;
                let info = unsafe {
                    sg::mtl::TextureInfo::new(Retained::as_ptr(&g.raw_texture) as sg::mtl::Handle)
                };
                let target =
                    sg::backend_render_targets::make_mtl((g.width as i32, g.height as i32), &info);
                let surface = sg::surfaces::wrap_backend_render_target(
                    &mut context,
                    &target,
                    sg::SurfaceOrigin::TopLeft,
                    sk::ColorType::BGRA8888,
                    None,
                    None,
                )
                .ok_or("Skia surface failed")?;
                Ok(Self::Skia(Skia {
                    context,
                    surface,
                    paths: vec![],
                    image: None,
                }))
            }
            "vello" => {
                let (renderer, resources) = vello_gpu::Renderer::new(
                    &g.device,
                    &vello_gpu::RenderTargetConfig {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        width: g.width as u16,
                        height: g.height as u16,
                    },
                );
                let depth = vello_gpu::Renderer::create_depth_texture_view(
                    &g.device,
                    &vello_gpu::RenderSize {
                        width: g.width as u16,
                        height: g.height as u16,
                    },
                );
                Ok(Self::Vello(Box::new(Vello {
                    renderer,
                    resources,
                    scene: vello_gpu::Scene::new(g.width as u16, g.height as u16),
                    paths: vec![],
                    depth,
                    image: None,
                    pending: None,
                })))
            }
            _ => Err("Unknown backend".into()),
        }
    }
    pub fn prepare(
        &mut self,
        f: &Fixture,
        g: &Gpu,
        frame: usize,
        changed: bool,
    ) -> Result<f64, String> {
        let timer = Instant::now();
        match self {
            Self::Skia(s) => {
                if changed {
                    s.paths = f
                        .paths
                        .iter()
                        .map(|p| {
                            let mut paint = sk::Paint::new(
                                sk::Color4f::new(
                                    p.rgba[0] as f32,
                                    p.rgba[1] as f32,
                                    p.rgba[2] as f32,
                                    p.rgba[3] as f32,
                                ),
                                None,
                            );
                            paint.set_anti_alias(true);
                            (sk_path(p), paint)
                        })
                        .collect();
                }
                if f.id == "image" && s.image.is_none() {
                    let info = sk::ImageInfo::new(
                        (2048, 2048),
                        sk::ColorType::RGBA8888,
                        sk::AlphaType::Unpremul,
                        None,
                    );
                    s.image =
                        sk::images::raster_from_data(&info, sk::Data::new_copy(&f.image), 2048 * 4);
                    if s.image.is_none() {
                        return Err("Skia image failed".into());
                    }
                }
            }
            Self::Vello(v) => {
                if changed {
                    v.paths = f
                        .paths
                        .iter()
                        .map(|p| {
                            (
                                bez_path(p),
                                AlphaColor::new([
                                    p.rgba[0] as f32,
                                    p.rgba[1] as f32,
                                    p.rgba[2] as f32,
                                    p.rgba[3] as f32,
                                ]),
                            )
                        })
                        .collect();
                }
                if f.id == "image" && v.image.is_none() {
                    let mut encoder = g.device.create_command_encoder(&Default::default());
                    let pixmap = Pixmap::from_parts(
                        f.image.clone(),
                        2048,
                        2048,
                        PixelMetadata::new(vello_common::peniko::ImageAlphaType::Alpha, true),
                    );
                    let id = v.renderer.upload_image(
                        &mut v.resources,
                        &g.device,
                        &g.queue,
                        &mut encoder,
                        &pixmap,
                    );
                    v.image = Some(
                        Image {
                            image: ImageSource::opaque_id(id),
                            sampler: Default::default(),
                        }
                        .with_quality(ImageQuality::Medium),
                    );
                    v.pending = Some(encoder);
                }
                if changed || f.id == "panzoom" || f.id == "image" {
                    v.scene.reset();
                    let base = Affine::new(transform(&f.id, frame, g.width, g.height));
                    v.scene.set_transform(base);
                    v.scene.set_fill_rule(Fill::NonZero);
                    if f.id == "clip" {
                        v.scene
                            .push_clip_rect(&Rect::new(150.0, 130.0, 1450.0, 870.0));
                        v.scene.push_clip_path(&clip_curve());
                    }
                    if let Some(image) = &v.image {
                        v.scene
                            .set_transform(base * Affine::new(image_transform(frame)));
                        v.scene.set_paint(image.clone());
                        v.scene.fill_rect(&Rect::new(0.0, 0.0, 2048.0, 2048.0));
                        v.scene.set_transform(base);
                    }
                    for (path, color) in &v.paths {
                        v.scene.set_paint(*color);
                        v.scene.fill_path(path);
                    }
                    if f.id == "clip" {
                        v.scene.pop_clip();
                        v.scene.pop_clip();
                    }
                }
            }
        }
        Ok(timer.elapsed().as_secs_f64() * 1000.0)
    }
    pub fn render(&mut self, f: &Fixture, g: &Gpu, frame: usize) -> Result<(), String> {
        match self {
            Self::Skia(s) => {
                let canvas = s.surface.canvas();
                canvas.clear(sk::Color::WHITE);
                canvas.save();
                let a = transform(&f.id, frame, g.width, g.height);
                canvas.concat(&matrix(a));
                if f.id == "clip" {
                    canvas.clip_rect(sk::Rect::new(150.0, 130.0, 1450.0, 870.0), None, true);
                    canvas.clip_path(&sk_from_bez(&clip_curve()), None, true);
                }
                if let Some(image) = &s.image {
                    canvas.save();
                    canvas.concat(&matrix(image_transform(frame)));
                    canvas.draw_image_with_sampling_options(
                        image,
                        (0.0, 0.0),
                        sk::SamplingOptions::new(sk::FilterMode::Linear, sk::MipmapMode::None),
                        None,
                    );
                    canvas.restore();
                }
                for (path, paint) in &s.paths {
                    canvas.draw_path(path, paint);
                }
                canvas.restore();
                s.context.flush_and_submit();
            }
            Self::Vello(v) => {
                let mut encoder = v
                    .pending
                    .take()
                    .unwrap_or_else(|| g.device.create_command_encoder(&Default::default()));
                v.renderer
                    .render(
                        &v.scene,
                        &mut v.resources,
                        &g.device,
                        &g.queue,
                        &mut encoder,
                        &vello_gpu::RenderSize {
                            width: g.width as u16,
                            height: g.height as u16,
                        },
                        &g.view,
                        Some(&v.depth),
                        &vello_gpu::TextureBindings::new(),
                        vello_gpu::TargetInit::Clear(vello_gpu::ClearSettings::Viewport {
                            color: AlphaColor::WHITE,
                        }),
                    )
                    .map_err(|e| e.to_string())?;
                g.queue.submit([encoder.finish()]);
            }
        }
        Ok(())
    }
}
fn matrix(a: [f64; 6]) -> sk::Matrix {
    sk::Matrix::new_all(
        a[0] as f32,
        a[2] as f32,
        a[4] as f32,
        a[1] as f32,
        a[3] as f32,
        a[5] as f32,
        0.0,
        0.0,
        1.0,
    )
}
fn bez_path(p: &Outline) -> BezPath {
    let mut path = BezPath::new();
    for c in &p.commands {
        let a = c.points;
        match c.verb {
            0 => path.move_to((a[0], a[1])),
            1 => path.line_to((a[0], a[1])),
            2 => path.curve_to((a[0], a[1]), (a[2], a[3]), (a[4], a[5])),
            _ => path.close_path(),
        }
    }
    path
}
fn sk_path(p: &Outline) -> sk::Path {
    sk_from_bez(&bez_path(p))
}
fn sk_from_bez(path: &BezPath) -> sk::Path {
    let mut p = sk::PathBuilder::new();
    for el in path.elements() {
        match *el {
            vello_common::kurbo::PathEl::MoveTo(a) => {
                p.move_to((a.x as f32, a.y as f32));
            }
            vello_common::kurbo::PathEl::LineTo(a) => {
                p.line_to((a.x as f32, a.y as f32));
            }
            vello_common::kurbo::PathEl::QuadTo(a, b) => {
                p.quad_to((a.x as f32, a.y as f32), (b.x as f32, b.y as f32));
            }
            vello_common::kurbo::PathEl::CurveTo(a, b, c) => {
                p.cubic_to(
                    (a.x as f32, a.y as f32),
                    (b.x as f32, b.y as f32),
                    (c.x as f32, c.y as f32),
                );
            }
            vello_common::kurbo::PathEl::ClosePath => {
                p.close();
            }
        }
    }
    p.detach()
}

#[cfg(test)]
mod tests {
    use super::parse_gpu_timing;
    #[test]
    fn profiling_mode_has_strict_opt_out() {
        assert!(parse_gpu_timing(None).unwrap());
        assert!(parse_gpu_timing(Some("on")).unwrap());
        assert!(!parse_gpu_timing(Some("off")).unwrap());
        for invalid in ["", "ON", "Off", "0", "false", " off", "off "] {
            assert!(
                parse_gpu_timing(Some(invalid)).is_err(),
                "accepted {invalid:?}"
            );
        }
    }
}
