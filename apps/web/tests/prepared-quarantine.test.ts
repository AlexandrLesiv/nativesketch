import test from 'node:test';
import assert from 'node:assert/strict';
import { setup } from './fixtures.ts';
import { Scene } from '../src/scene.ts';
await test('injected copy failure after consumed prepared release quarantines without repeating the committed command', async () => {
  const { engine } = await setup(),
    scene = new Scene(engine);
  engine.begin(1n, 2);
  engine.append(1n, [
    { x: -1e8, y: 40, time: 0, id: 1n },
    { x: 1e8, y: 40, time: 1, id: 2n },
  ]);
  engine.commit(1n);
  const frame = scene.update([0, 0, 100, 100, 2]);
  const call = engine.transport.call.bind(engine.transport),
    invoke = engine.transport.invoke.bind(engine.transport);
  let releasing = false,
    consumed = false,
    committedCalls = 0;
  engine.transport.call = (op, body) => {
    if (op === 47) releasing = true;
    return call(op, body);
  };
  engine.transport.invoke = (name, args) => {
    if (consumed && name === 'buffer_len') throw Error('injected post-release copy failure');
    const result = invoke(name, args);
    if (releasing && name === 'app_call') {
      consumed = true;
      committedCalls++;
    }
    return result;
  };
  assert.throws(() => frame.prepare!(frame.strokes[0], 133, 137, 100000), /post-release/);
  assert.equal(engine.isAvailable(), false);
  assert.throws(() => scene.dispose(), /unavailable/);
  assert.throws(() => scene.dispose(), /unavailable/);
  assert.equal(committedCalls, 1);
  // A quarantined instance is discarded; cleanup exports must not run after uncertain copying.
});
