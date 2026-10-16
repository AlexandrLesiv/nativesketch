import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { setup, draw } from './fixtures.ts';
import { Scene } from '../src/scene.ts';
import { Engine } from '../src/engine.ts';
await test('injected scene page failure retains coherent previous frame and retires staging lease', async () => {
  const { engine: e, work } = await setup();
  draw(e);
  const scene = new Scene(e),
    viewport = [0, 0, 500, 500, 32];
  const before = scene.update(viewport);
  draw(e, 9n, 90);
  const original = e.transport.call.bind(e.transport);
  e.transport.call = (op, body) => {
    if (op === 43) throw Error('injected page read failure');
    return original(op, body);
  };
  assert.throws(() => scene.update(viewport), /injected/);
  assert.equal(scene.frame, before);
  e.transport.call = original;
  assert.equal(scene.update(viewport).strokes.length, 2);
  scene.dispose();
  await work.shutdown();
});
await test('injected genuine Wasm trap quarantines transport and does not call cleanup exports', async () => {
  const { instance } = await WebAssembly.instantiate(
    await readFile('artifacts/web/sketch_app_wasm.wasm'),
  );
  const trap = await WebAssembly.instantiate(
    new Uint8Array([
      0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 7, 8, 1, 4, 116, 114, 97, 112, 0,
      0, 10, 5, 1, 3, 0, 0, 11,
    ]),
  );
  let enabled = false,
    calls = 0;
  const exports = {
    ...instance.exports,
    ns_v2_wasm_app_call: (...args: number[]) => {
      calls++;
      if (enabled) (trap.instance.exports.trap as CallableFunction)();
      return (instance.exports.ns_v2_wasm_app_call as CallableFunction)(...args);
    },
  };
  const e = new Engine(exports);
  enabled = true;
  assert.throws(() => e.state(), WebAssembly.RuntimeError);
  const count = calls;
  assert.equal(e.isAvailable(), false);
  assert.throws(() => e.state(), /unavailable/);
  assert.equal(calls, count);
});
await test('injected capacity rejection after actual Finish retains result until controlled shutdown', async () => {
  const { engine: e, work } = await setup();
  draw(e);
  const { u64 } = await import('../src/wire.ts');
  const { EngineError } = await import('../src/transport.ts');
  work.supply(u64(e.file(3), 240));
  const original = e.transport.invoke.bind(e.transport);
  let armed = false,
    rejected = false,
    finishes = 0,
    downloads = 0;
  e.transport.invoke = (name, args) => {
    if (armed && !rejected && name === 'upload_new') {
      rejected = true;
      throw new EngineError(3);
    }
    const receipt = original(name, args);
    if (name === 'work_call' && receipt.flags & 1) {
      armed = true;
      finishes++;
    }
    return receipt;
  };
  const failures: unknown[] = [];
  await work.drain(
    () => {},
    () => {
      downloads++;
    },
    (error) => failures.push(error),
  );
  assert.equal(rejected, true);
  assert.equal(failures.length, 1);
  assert.equal(finishes, 1);
  assert.equal(downloads, 1);
  e.transport.invoke = original;
  await work.shutdown();
  assert.equal(e.isAvailable(), false);
});
