import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout } from 'node:timers/promises';
import { spawn } from 'node:child_process';
import { fixture } from '../workbench/tests/fixture.ts';
import { runOwned } from '../workbench/process.ts';
import { executeNode } from '../evidence/node.ts';
const runUrl = new URL('../lib/process.ts', import.meta.url).href;
const ownedUrl = new URL('../workbench/process.ts', import.meta.url).href;
async function pidAt(path: string) {
  for (let i = 0; i < 200; i++) {
    try {
      return Number(await readFile(path, 'utf8'));
    } catch {
      await setTimeout(10);
    }
  }
  throw new Error('Disposable descendant never became ready');
}
async function gone(pid: number) {
  for (let i = 0; i < 200; i++) {
    try {
      process.kill(pid, 0);
    } catch (error) {
      assert.equal((error as NodeJS.ErrnoException).code, 'ESRCH');
      return;
    }
    await setTimeout(10);
  }
  assert.fail('Owned descendant remained alive');
}
async function tree() {
  const f = await fixture();
  await f.put(
    'leaf.ts',
    `import {writeFileSync} from 'node:fs'; process.on('SIGTERM',()=>{});writeFileSync('artifacts/leaf.pid',String(process.pid));if(process.argv[2]==='flood')process.stdout.write('x'.repeat(2000000));setInterval(()=>{},1000);`,
  );
  await f.put(
    'middle.ts',
    `import {run} from ${JSON.stringify(runUrl)};process.on('SIGTERM',()=>{});await run(process.execPath,['leaf.ts',process.argv[2]??''],{cwd:process.cwd(),capture:false});`,
  );
  return f;
}
await test(
  'owned tree executeNode cancellation removes nested runOwned and run descendants',
  { skip: process.platform === 'win32' },
  async () => {
    const f = await tree();
    const controller = new AbortController();
    const sentinel = spawn(process.execPath, ['-e', 'setTimeout(()=>{},30000)'], {
      stdio: 'ignore',
      detached: true,
    });
    try {
      await f.put(
        'nested.test.ts',
        `import test from 'node:test';import {runOwned} from ${JSON.stringify(ownedUrl)};await test('nested',async()=>{await runOwned(process.execPath,['middle.ts'],process.cwd(),new AbortController().signal,()=>{});});`,
      );
      const pending = executeNode(f.root, ['nested.test.ts'], controller.signal);
      const rejected = assert.rejects(pending, /cancelled/);
      const pid = await pidAt(join(f.root, 'artifacts/leaf.pid'));
      controller.abort();
      await rejected;
      await gone(pid);
      assert.ok(sentinel.pid);
      assert.doesNotThrow(() => process.kill(sentinel.pid!, 0));
    } finally {
      controller.abort();
      sentinel.kill('SIGKILL');
      await f.close();
    }
  },
);
await test(
  'owned tree runOwned timeout and output flood remove nested run descendants',
  { skip: process.platform === 'win32' },
  async () => {
    for (const mode of ['timeout', 'flood']) {
      const f = await tree();
      try {
        const pending = runOwned(
          process.execPath,
          ['middle.ts', mode],
          f.root,
          new AbortController().signal,
          () => {},
          mode === 'timeout' ? 1000 : 5000,
        );
        const pid = await pidAt(join(f.root, 'artifacts/leaf.pid'));
        const result = await pending;
        assert.equal(result.reason, mode === 'timeout' ? 'Timed out' : 'Output exceeded 1 MiB');
        await gone(pid);
      } finally {
        await f.close();
      }
    }
  },
);
await test(
  'owned tree nested timeout preserves sibling work and exact successful outer exit',
  { skip: process.platform === 'win32' },
  async () => {
    const f = await tree();
    try {
      await f.put(
        'siblings.ts',
        `import {run} from ${JSON.stringify(runUrl)};
await Promise.all([run(process.execPath,['leaf.ts'],{cwd:process.cwd(),timeout:200,capture:true}).catch(error=>{if(!error.message.includes('timed out'))throw error;}),run(process.execPath,['-e','setTimeout(()=>console.log("sibling survived"),500)'],{cwd:process.cwd(),capture:true}).then(text=>console.log(text))]);`,
      );
      const result = await runOwned(
        process.execPath,
        ['siblings.ts'],
        f.root,
        new AbortController().signal,
        () => {},
      );
      assert.equal(result.exitCode, 0);
      assert.equal(result.signal, null);
      assert.equal(result.reason, null);
      assert.match(result.output, /sibling survived/);
      await gone(await pidAt(join(f.root, 'artifacts/leaf.pid')));
    } finally {
      await f.close();
    }
  },
);
await test(
  'owned tree launch failure and pre-cancel cannot become successful execution',
  { skip: process.platform === 'win32' },
  async () => {
    const f = await fixture();
    try {
      const failed = await runOwned(
        '/missing/native-sketch-executable',
        [],
        f.root,
        new AbortController().signal,
        () => {},
      );
      assert.match(failed.reason ?? '', /failed to spawn|unavailable/);
      const controller = new AbortController();
      controller.abort();
      const cancelled = await runOwned(
        process.execPath,
        ['-e', 'process.exit(0)'],
        f.root,
        controller.signal,
        () => {},
      );
      assert.equal(cancelled.reason, 'Cancelled');
      assert.equal(cancelled.exitCode, null);
    } finally {
      await f.close();
    }
  },
);
