import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, access } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout } from 'node:timers/promises';
import { fixture } from '../workbench/tests/fixture.ts';
import { executeNode } from '../evidence/node.ts';
import { backendEvidence } from '../workbench-e2e/backend.ts';
import { sourceIdentity } from '../evidence/identity.ts';
await test('Node evidence pre-abort prevents backend spawn and passing artifacts', async () => {
  const f = await fixture();
  try {
    const controller = new AbortController();
    controller.abort();
    await assert.rejects(
      backendEvidence(
        f.root,
        join(f.root, 'artifacts'),
        await sourceIdentity(f.root),
        1,
        controller.signal,
      ),
      /abort/i,
    );
    await assert.rejects(access(join(f.root, 'artifacts/backend-results.json')));
    await assert.rejects(executeNode(f.root, ['missing.test.ts'], controller.signal), /abort/i);
  } finally {
    await f.close();
  }
});
await test(
  'Node evidence cancellation reaps delayed worker ignoring SIGTERM',
  { skip: process.platform === 'win32' },
  async () => {
    const f = await fixture();
    try {
      await f.put(
        'scripts/workbench/tests/delayed.test.ts',
        `import test from 'node:test'; import {writeFileSync} from 'node:fs';
process.on('SIGTERM',()=>{});
await test('delayed',async()=>{writeFileSync('artifacts/worker.pid',String(process.pid));await new Promise(r=>setTimeout(r,30000));});`,
      );
      const controller = new AbortController();
      const started = Date.now();
      const pending = backendEvidence(
        f.root,
        join(f.root, 'artifacts'),
        await sourceIdentity(f.root),
        1,
        controller.signal,
      );
      const rejected = assert.rejects(pending, /cancelled/);
      let pid = 0;
      for (let i = 0; i < 100; i++) {
        try {
          pid = Number(await readFile(join(f.root, 'artifacts/worker.pid'), 'utf8'));
          break;
        } catch {
          await setTimeout(20);
        }
      }
      assert.ok(pid > 0);
      controller.abort();
      await rejected;
      assert.ok(Date.now() - started < 5000);
      await assert.rejects(access(join(f.root, 'artifacts/backend-results.json')));
      let alive = true;
      for (let i = 0; i < 100 && alive; i++) {
        try {
          process.kill(pid, 0);
          await setTimeout(20);
        } catch (error) {
          assert.equal((error as NodeJS.ErrnoException).code, 'ESRCH');
          alive = false;
        }
      }
      assert.equal(alive, false, 'owned test worker must no longer exist');
    } finally {
      await f.close();
    }
  },
);
