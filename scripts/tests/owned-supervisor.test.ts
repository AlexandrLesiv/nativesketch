import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout } from 'node:timers/promises';
import { fixture } from '../workbench/tests/fixture.ts';
import { spawnOwned } from '../lib/owned-process.ts';
await test(
  'owned tree abrupt supervisor death still removes registered descendants',
  { skip: process.platform === 'win32' },
  async () => {
    const f = await fixture();
    const runUrl = new URL('../lib/process.ts', import.meta.url).href;
    let owned: ReturnType<typeof spawnOwned> | null = null;
    try {
      await f.put(
        'leaf.ts',
        `import {writeFileSync} from 'node:fs';process.on('SIGTERM',()=>{});writeFileSync('artifacts/leaf.pid',String(process.pid));setInterval(()=>{},1000);`,
      );
      await f.put(
        'parent.ts',
        `import {run} from ${JSON.stringify(runUrl)};process.on('SIGTERM',()=>{});await run(process.execPath,['leaf.ts'],{cwd:process.cwd()});`,
      );
      owned = spawnOwned(process.execPath, ['parent.ts'], { cwd: f.root });
      const running = owned;
      const closed = new Promise<{ code: number | null; signal: string | null; error: string }>(
        (resolve) =>
          running.child.on('close', (code, signal) => resolve(running.finish(code, signal))),
      );
      let pid = 0;
      for (let i = 0; i < 200 && !pid; i++) {
        try {
          pid = Number(await readFile(join(f.root, 'artifacts/leaf.pid'), 'utf8'));
        } catch {
          await setTimeout(10);
        }
      }
      assert.ok(pid > 0);
      owned.child.kill('SIGKILL');
      const result = await Promise.race([
        closed,
        setTimeout(5000, undefined, { ref: false }).then(() => {
          throw new Error('Supervisor cleanup exceeded bound');
        }),
      ]);
      assert.equal(result.signal, 'SIGKILL');
      assert.equal(result.code, null);
      assert.equal(result.error, '');
      let alive = true;
      for (let i = 0; i < 200 && alive; i++) {
        try {
          process.kill(pid, 0);
          await setTimeout(10);
        } catch (error) {
          assert.equal((error as NodeJS.ErrnoException).code, 'ESRCH');
          alive = false;
        }
      }
      assert.equal(alive, false);
    } finally {
      owned?.kill('SIGKILL');
      await f.close();
    }
  },
);
