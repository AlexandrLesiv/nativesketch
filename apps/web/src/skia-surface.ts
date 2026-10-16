import type { CanvasKit, Surface, GrDirectContext, MallocObj } from 'canvaskit-wasm';

/** One DOM canvas; raster is admitted only before any successful GL acquisition. */
export class SkiaSurface {
  readonly kit: CanvasKit;
  readonly element: HTMLCanvasElement;
  backend: 'webgl' | 'raster';
  private handle = 0;
  private context: GrDirectContext | null = null;
  private surface: Surface | null = null;
  private pixels: MallocObj | undefined;
  private raster: CanvasRenderingContext2D | null = null;
  private width = 0;
  private height = 0;
  constructor(kit: CanvasKit, element: HTMLCanvasElement) {
    this.kit = kit;
    this.element = element;
    this.handle = kit.GetWebGLContext(element, { alpha: 0, antialias: 1 });
    this.backend = this.handle ? 'webgl' : 'raster';
    try {
      if (this.handle) this.createContext();
      else {
        this.raster = element.getContext('2d');
        if (!this.raster) throw Error('Raster presentation unavailable');
      }
    } catch (error) {
      this.dispose();
      throw error;
    }
  }
  private createContext() {
    this.context = this.kit.MakeWebGLContext(this.handle);
    if (!this.context) throw Error('WebGL renderer unavailable');
    this.context.setResourceCacheLimitBytes(32 * 1024 * 1024);
  }
  private releaseSurface() {
    this.surface?.delete();
    this.surface = null;
    if (this.pixels) this.kit.Free(this.pixels);
    this.pixels = undefined;
    this.width = this.height = 0;
  }
  acquire() {
    const { width, height } = this.element;
    if (!width || !height || width * height > 16 * 1024 * 1024)
      throw Error('Canvas exceeds the renderer pixel budget');
    if (this.surface && width === this.width && height === this.height) return this.surface;
    this.releaseSurface();
    if (this.backend === 'webgl') {
      if (!this.context) throw Error('WebGL context is unavailable');
      this.surface = this.kit.MakeOnScreenGLSurface(
        this.context,
        width,
        height,
        this.kit.ColorSpace.SRGB,
      );
    } else {
      this.pixels = this.kit.Malloc(Uint8Array, width * height * 4);
      try {
        this.surface = this.kit.MakeRasterDirectSurface(
          {
            width,
            height,
            colorType: this.kit.ColorType.RGBA_8888,
            alphaType: this.kit.AlphaType.Premul,
            colorSpace: this.kit.ColorSpace.SRGB,
          },
          this.pixels,
          width * 4,
        );
      } catch (error) {
        this.releaseSurface();
        throw error;
      }
    }
    if (!this.surface) {
      this.releaseSurface();
      throw Error('Renderer surface allocation failed');
    }
    this.width = width;
    this.height = height;
    return this.surface;
  }
  present() {
    if (this.pixels && this.raster) {
      const bytes = new Uint8ClampedArray(this.pixels.toTypedArray());
      this.raster.putImageData(new ImageData(bytes, this.width, this.height), 0, 0);
    }
  }
  lost() {
    this.releaseSurface();
    this.context?.releaseResourcesAndAbandonContext();
    this.context?.delete();
    this.context = null;
  }
  restore() {
    if (this.backend !== 'webgl') return;
    this.createContext();
  }
  dispose() {
    this.lost();
    if (this.handle) this.kit.deleteContext(this.handle);
    this.handle = 0;
  }
}
