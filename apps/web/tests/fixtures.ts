import { readFile } from 'node:fs/promises';
import { Engine } from '../src/engine.ts';
import { FileWork } from '../src/file-work.ts';
import { u32, u64 } from '../src/wire.ts';
export async function setup() {
  const { instance } = await WebAssembly.instantiate(
    await readFile('artifacts/web/sketch_app_wasm.wasm'),
  );
  const engine = new Engine(instance.exports),
    work = new FileWork(engine);
  return { engine, work };
}
export function draw(e: Engine, pointer = 0xfedcba9876543210n, offset = 0) {
  e.action('pen');
  e.begin(pointer, 2);
  e.append(pointer, [
    { x: 20 + offset, y: 20, time: 0, id: 0xfedcba9876543211n, pressure: 0.7 },
    { x: 80 + offset, y: 80, time: 1, id: 0xfedcba9876543212n },
  ]);
  e.commit(pointer);
}
export async function settle(w: FileWork, dispatch = (_b: Uint8Array) => {}) {
  const effects: Uint8Array[] = [],
    failures: unknown[] = [];
  await w.drain(
    (b) => effects.push(b),
    dispatch,
    (e) => failures.push(e),
  );
  if (failures.length) throw failures[0];
  return effects;
}
export async function exported(e: Engine, w: FileWork) {
  w.supply(u64(e.file(3), 240));
  let bytes: Uint8Array = new Uint8Array();
  await settle(w, (b) => {
    bytes = b;
  });
  return bytes;
}
export function open(
  e: Engine,
  w: FileWork,
  source: { size: number; arrayBuffer(): Promise<ArrayBuffer> },
) {
  let effect = e.file(5);
  if (u32(effect, 232) === 3) effect = e.reply(u64(effect, 240), 3);
  return w.supply(u64(effect, 240), source);
}
