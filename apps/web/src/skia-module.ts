import publishedModule from 'canvaskit-wasm/full';
import type { CanvasKit, CanvasKitInitOptions } from 'canvaskit-wasm';
/** 0.42 ships a CJS callable with ES-default declarations; normalize that NodeNext boundary. */
type Initializer = (
  options: CanvasKitInitOptions & { wasmBinary: Uint8Array },
) => Promise<CanvasKit>;
const initializer: unknown = publishedModule;
if (typeof initializer !== 'function') throw Error('Invalid pinned CanvasKit initializer');
export const initializeSkia = initializer as Initializer;
