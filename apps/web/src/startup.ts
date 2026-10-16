import type { EditorHost } from './host.ts';
import { Scene } from './scene.ts';
import { Engine } from './engine.ts';
import { startRenderer } from './renderer.ts';
export async function startEngine(app: EditorHost) {
  try {
    const response = await fetch('sketch_app_wasm.wasm');
    if (!response.ok) throw Error('Drawing engine unavailable');
    const instance = await WebAssembly.instantiate(await response.arrayBuffer(), {});
    app.engine = new Engine(instance.instance.exports);
    app.scene = new Scene(app.engine);
    try {
      app.renderer = await startRenderer(app);
    } catch (error) {
      app.canvas.dataset.rendererError = String(error);
    }
    app.refresh();
    if ('serviceWorker' in navigator)
      void navigator.serviceWorker.register('./sw.js').catch(() => {});
  } catch (e) {
    app.disableEngine('Drawing engine failed to load. Reload to try again.', false);
    app.error(e);
  }
}
