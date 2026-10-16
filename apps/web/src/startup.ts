import type { EditorHost } from './host.ts';
import { Engine } from './engine.ts';
import type { Api } from './engine.ts';
export async function startEngine(app: EditorHost) {
  try {
    const response = await fetch('sketch_wasm.wasm');
    if (!response.ok) throw Error('Drawing engine unavailable');
    const instance = await WebAssembly.instantiate(await response.arrayBuffer(), {});
    app.engine = new Engine(instance.instance.exports as Api);
    app.refresh();
    if ('serviceWorker' in navigator)
      void navigator.serviceWorker.register('./sw.js').catch(() => {});
  } catch (e) {
    app.disableEngine('Drawing engine failed to load. Reload to try again.', false);
    app.error(e);
  }
}
