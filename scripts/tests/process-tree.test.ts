import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, readFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout } from 'node:timers/promises';
import { run } from '../lib/process.ts';
import { root } from '../lib/context.ts';
await test(
  'cancellation kills a descendant that ignores TERM after its parent exits',
  { skip: process.platform === 'win32' },
  async (t) => {
    await mkdir(join(root, 'artifacts/tests'), { recursive: true });
    const directory = await mkdtemp(join(root, 'artifacts/tests/process-tree-'));
    t.after(() => rm(directory, { recursive: true, force: true }));
    const heartbeat = join(directory, 'heartbeat');
    const descendant = `process.on('SIGTERM',()=>{});setInterval(()=>require('fs').appendFileSync(process.argv[1],'x'),20)`;
    const parent = `require('child_process').spawn(process.execPath,['-e',${JSON.stringify(descendant)},process.argv[1]],{stdio:'ignore'});setInterval(()=>{},1000)`;
    const controller = new AbortController();
    const result = run(process.execPath, ['-e', parent, heartbeat], {
      cwd: directory,
      signal: controller.signal,
    });
    // Attach rejection immediately while waiting for a real descendant heartbeat.
    const rejected = assert.rejects(result, /cancelled/);
    let ready = false;
    for (let attempt = 0; attempt < 100; attempt++) {
      try {
        if ((await readFile(heartbeat)).length > 0) {
          ready = true;
          break;
        }
      } catch {}
      await setTimeout(20);
    }
    controller.abort();
    await rejected;
    assert.ok(ready, 'descendant did not start');
    await setTimeout(100);
    const stopped = (await readFile(heartbeat)).length;
    await setTimeout(200);
    assert.equal((await readFile(heartbeat)).length, stopped, 'descendant continued after cleanup');
  },
);
