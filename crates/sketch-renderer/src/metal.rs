use crate::{Compositor, RenderError, RenderRequest, RenderStats};
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{MTLCommandQueue, MTLPixelFormat, MTLResource, MTLTexture};
use sketch_core::{Bounds, RenderScene};
use skia_safe::{
    ColorSpace, ColorType,
    gpu::{self, DirectContext, SurfaceOrigin, mtl},
};
use std::ffi::c_void;
/// Call only on the creating thread; the FFI wrapper enforces affinity.
pub struct MetalRenderer {
    compositor: Compositor,
    context: DirectContext,
    backend: mtl::BackendContext,
    device: *const c_void,
}
impl MetalRenderer {
    /// # Safety
    /// Both pointers are valid Metal objects; queue belongs to device.
    pub unsafe fn new(device: *const c_void, queue: *const c_void) -> Result<Self, RenderError> {
        if device.is_null() || queue.is_null() {
            return Err(RenderError::InvalidRequest);
        }
        let queue_object = unsafe { &*queue.cast::<ProtocolObject<dyn MTLCommandQueue>>() };
        if Retained::as_ptr(&queue_object.device()).cast::<c_void>() != device {
            return Err(RenderError::InvalidRequest);
        }
        let backend = unsafe { mtl::BackendContext::new(device, queue) };
        let mut context =
            gpu::direct_contexts::make_metal(&backend, None).ok_or(RenderError::Backend)?;
        context.set_resource_cache_limit(64 * 1024 * 1024);
        Ok(Self {
            compositor: Compositor::default(),
            context,
            backend,
            device,
        })
    }
    pub fn validate(&self, request: &RenderRequest) -> Result<(), RenderError> {
        self.compositor.validate(request)
    }
    /// Reconstructs the actual Metal context, not just cache keys.
    pub fn reset(&mut self) -> Result<(), RenderError> {
        let mut context =
            gpu::direct_contexts::make_metal(&self.backend, None).ok_or(RenderError::Backend)?;
        context.set_resource_cache_limit(64 * 1024 * 1024);
        self.compositor.reset()?;
        self.context = context;
        Ok(())
    }
    /// # Safety
    /// Texture points to a valid Metal texture; this validates metadata without GPU work.
    pub unsafe fn validate_texture(
        &self,
        texture: *const c_void,
        request: &RenderRequest,
    ) -> Result<(), RenderError> {
        self.validate(request)?;
        if texture.is_null() {
            return Err(RenderError::InvalidRequest);
        }
        let object = unsafe { &*texture.cast::<ProtocolObject<dyn MTLTexture>>() };
        if object.width() != request.pixel_width as usize
            || object.height() != request.pixel_height as usize
            || object.sampleCount() != 1
            || object.pixelFormat() != MTLPixelFormat::BGRA8Unorm
            || Retained::as_ptr(&object.device()).cast::<c_void>() != self.device
        {
            return Err(RenderError::InvalidRequest);
        }
        Ok(())
    }
    /// # Safety
    /// Texture is a valid single-sample Metal drawable texture, kept alive through submission.
    pub unsafe fn draw(
        &mut self,
        texture: *const c_void,
        scene: &RenderScene,
        request: &RenderRequest,
        selected: Option<Bounds>,
    ) -> Result<RenderStats, RenderError> {
        unsafe {
            self.validate_texture(texture, request)?;
        }
        if !request.matches(scene.key) {
            return Err(RenderError::Stale);
        }
        let info = unsafe { mtl::TextureInfo::new(texture) };
        let target = gpu::backend_render_targets::make_mtl(
            (request.pixel_width as i32, request.pixel_height as i32),
            &info,
        );
        let mut surface = gpu::surfaces::wrap_backend_render_target(
            &mut self.context,
            &target,
            SurfaceOrigin::TopLeft,
            ColorType::BGRA8888,
            ColorSpace::new_srgb(),
            None,
        )
        .ok_or(RenderError::Backend)?;
        let stats = self
            .compositor
            .render(&mut surface, scene, request, selected, true)?;
        self.context.flush(None);
        if !self.context.submit(gpu::SyncCpu::No) {
            return Err(RenderError::Backend);
        }
        Ok(stats)
    }
}
