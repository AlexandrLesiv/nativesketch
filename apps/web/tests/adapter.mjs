import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
// .js emitted outside package scope is ESM via data URL for the Node probe.
const source = await readFile("artifacts/web/engine.js", "utf8");
const { Engine, identity, floats } = await import(
    `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`
);
const { instance } = await WebAssembly.instantiate(
    await readFile("artifacts/web/sketch_wasm.wasm"),
    {},
);
const e = new Engine(instance.exports);
e.begin(9007199254740995n, 1, [0.2, 0.3, 0.9, 0.5], 4);
e.append(9007199254740995n, [
    { x: 20, y: 20, time: 0, id: 1n, pressure: 0.7 },
    { x: 80, y: 80, time: 1, id: 2n },
]);
e.command(4, identity(9007199254740995n));
assert.equal(e.state().count, 1n);
assert.equal(e.state().dirty, true);
const snapshot = e.snapshot();
assert.equal(
    e.state().dirty,
    true,
    "export/download cannot confirm persistence",
);
e.begin(2n, 2, [0, 0, 0, 1], 8);
e.append(2n, [{ x: 100, y: 100, time: 0, id: 1n }]);
e.command(5);
assert.equal(
    e.state().count,
    1n,
    "second-contact/focus cancellation keeps committed strokes",
);
e.cycle(1);
e.command(11);
e.command(12, floats(2, 0, 0));
e.command(13);
assert.throws(
    () => e.command(19, identity(snapshot.revision)),
    "stale completion must not mark edited document clean",
);
const changed = e.snapshot();
assert.throws(() => e.command(18, new Uint8Array([1, 2, 3])));
assert.deepEqual(e.snapshot(), changed);
e.command(14);
assert.deepEqual(e.snapshot().bytes, snapshot.bytes);
e.command(15);
assert.deepEqual(e.snapshot().bytes, changed.bytes);
e.command(10);
assert.equal(e.state().count, 0n);
e.command(14);
assert.equal(e.state().count, 1n);
e.command(18, snapshot.bytes);
assert.equal(e.state().dirty, false);
assert.equal(e.state().undo, false);
const render = e.command(6);
assert.ok(render.length > 0);
assert.equal(render.length % 96, 0);
e.dispose();
assert.throws(() => e.state());
console.log(
    "Browser adapter: actual Wasm identity, ink/cancel, resize/history/delete, invalid import, snapshot/dirty checks passed.",
);
