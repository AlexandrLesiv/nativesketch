import test from 'node:test';
import assert from 'node:assert/strict';
import { setTimeout } from 'node:timers/promises';
import { readFile, readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { fixture } from './fixture.ts';
import { Jobs } from '../jobs.ts';
import { Previews } from '../preview.ts';
import { sourceIdentity } from '../identity.ts';
await test('workbench completion remains running and cancellation cannot become passed', async () => {
  const f = await fixture();
  await f.put('scripts/check.ts', 'process.exit(0);');
  let entered = false;
  let release: () => void = () => {};
  const paused = new Promise<void>((resolve) => {
    release = resolve;
  });
  const jobs = new Jobs(f.root, async (_job, signal) => {
    entered = true;
    await paused;
    signal.throwIfAborted();
  });
  try {
    const job = await jobs.enqueue('check-core');
    for (let i = 0; i < 100 && !entered; i++) await setTimeout(20);
    assert.equal(entered, true);
    assert.equal(jobs.list()[0]?.state, 'running');
    jobs.cancel(job.id);
    release();
    await jobs.close();
    assert.equal(jobs.list()[0]?.state, 'cancelled');
    assert.equal(jobs.list()[0]?.reason, 'Cancelled');
  } finally {
    release();
    await jobs.close();
    await f.close();
  }
});
await test('workbench previews serve frozen bytes and preserve old preview after failed publication', async () => {
  const f = await fixture();
  const previews = new Previews();
  try {
    await f.put('artifacts/workbench/previews/.keep', '');
    await f.put('artifacts/web/index.html', 'original');
    await f.put('artifacts/web/app.wasm', 'fixture wasm bytes');
    const source = await sourceIdentity(f.root);
    await previews.publish(f.root, 'first', source, new AbortController().signal);
    const original = previews.current;
    assert.ok(original);
    await f.put('artifacts/web/index.html', 'changed source');
    await f.put('artifacts/workbench/previews/first/index.html', 'changed copy');
    assert.equal(await (await fetch(original.url)).text(), 'original');
    const controller = new AbortController();
    controller.abort();
    await assert.rejects(previews.publish(f.root, 'cancelled', source, controller.signal));
    assert.deepEqual(previews.current, original);
    assert.ok(!(await readdir(join(f.root, 'artifacts/workbench/previews'))).includes('cancelled'));
    await f.put('changed.txt', 'source drift');
    await assert.rejects(
      previews.publish(f.root, 'stale', source, new AbortController().signal),
      /Source changed/,
    );
    assert.deepEqual(previews.current, original);
    assert.equal(
      (await readFile(join(f.root, 'artifacts/workbench/previews/first/app.wasm'))).toString(),
      'fixture wasm bytes',
    );
  } finally {
    await previews.close();
    await f.close();
  }
});
