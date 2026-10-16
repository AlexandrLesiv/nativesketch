import { initializeSkia } from './skia-module.ts';
import type { CanvasKit } from 'canvaskit-wasm';
let pending: Promise<CanvasKit> | undefined;
/** The exact full-bundle Wasm is local and included in the offline asset manifest. */
export function loadSkia() {
  pending ??= (async () => {
    const response = await fetch(new URL('./canvaskit.wasm', document.baseURI), {
      signal: AbortSignal.timeout(15000),
    });
    if (!response.ok) throw Error('Renderer download failed');
    const wasmBinary = new Uint8Array(await response.arrayBuffer());
    const options = {
      wasmBinary,
      locateFile: () => new URL('./canvaskit.wasm', document.baseURI).href,
    };
    return initializeSkia(options);
  })();
  return pending;
}
