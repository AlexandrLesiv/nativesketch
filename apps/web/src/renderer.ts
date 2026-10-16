import type { EditorHost } from './host.ts';
import { loadSkia } from './skia-loader.ts';
import { SkiaSurface } from './skia-surface.ts';
import { SkiaPainter } from './skia-painter.ts';
import { el } from './elements.ts';

export class Renderer {
  private app: EditorHost;
  private surface: SkiaSurface;
  private painter: SkiaPainter;
  available = true;
  private contextLost = false;
  private disposed = false;
  private onPageHide = (event: PageTransitionEvent) => {
    if (!event.persisted) this.dispose();
  };
  private onLost = (event: Event) => {
    event.preventDefault();
    if (!this.available) return;
    this.available = false;
    this.contextLost = true;
    this.app.cancel();
    for (const id of this.app.contacts.keys())
      if (this.app.canvas.hasPointerCapture(id)) this.app.canvas.releasePointerCapture(id);
    this.app.contacts.clear();
    this.app.pair = [];
    this.app.navigation = undefined;
    this.app.space = false;
    this.surface.lost();
    this.app.refresh();
  };
  private onRestored = () => {
    if (!this.contextLost) return;
    this.contextLost = false;
    try {
      this.surface.restore();
      this.available = true;
      this.app.refresh();
    } catch (error) {
      this.fail(error);
    }
  };
  constructor(app: EditorHost, surface: SkiaSurface, painter: SkiaPainter) {
    this.app = app;
    this.surface = surface;
    this.painter = painter;
    app.canvas.dataset.renderer = `CanvasKit 0.42.0 ${surface.backend}`;
    window.addEventListener('pagehide', this.onPageHide);
    app.canvas.addEventListener('webglcontextlost', this.onLost);
    app.canvas.addEventListener('webglcontextrestored', this.onRestored);
  }
  paint() {
    if (!this.available) return;
    const app = this.app,
      x = -app.tx / app.zoom,
      y = -app.ty / app.zoom;
    const frame = app.scene.update([
      x,
      y,
      app.width / app.zoom,
      app.height / app.zoom,
      32 / app.zoom,
    ]);
    const state = app.engine.state();
    this.painter.draw(
      this.surface.acquire(),
      frame,
      { x, y, zoom: app.zoom, dpr: window.devicePixelRatio || 1 },
      state.selected ? state.bounds : undefined,
    );
    this.surface.present();
  }
  fail(error: unknown) {
    this.available = false;
    this.app.cancel();
    this.app.canvas.dataset.rendererError = String(error);
    el('status').textContent += ' · Rendering unavailable; download remains available';
    this.app.canvas.setAttribute('aria-disabled', 'true');
  }
  dispose() {
    if (this.disposed) return;
    this.disposed = true;
    this.available = false;
    window.removeEventListener('pagehide', this.onPageHide);
    this.app.canvas.removeEventListener('webglcontextlost', this.onLost);
    this.app.canvas.removeEventListener('webglcontextrestored', this.onRestored);
    this.painter.dispose();
    this.surface.dispose();
  }
}
export async function startRenderer(app: EditorHost) {
  const kit = await loadSkia();
  const surface = new SkiaSurface(kit, app.canvas);
  try {
    return new Renderer(app, surface, new SkiaPainter(kit));
  } catch (error) {
    surface.dispose();
    throw error;
  }
}
