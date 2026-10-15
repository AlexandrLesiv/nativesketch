import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { run } from '../lib/process.ts';
import { root } from '../lib/context.ts';
await test('subprocess lifecycle', async (t) => {
  await mkdir(join(root, 'artifacts/tests'), { recursive: true });
  const fixture = await mkdtemp(join(root, 'artifacts/tests/process spaces Україна-'));
  const node = process.execPath;
  t.after(async () => rm(fixture, { recursive: true, force: true }));
  await t.test('argv preserves spaces, unicode and shell metacharacters', async () => {
    const input = 'a b Україна $(echo unsafe) & "';
    assert.equal(
      await run(node, ['-e', 'process.stdout.write(process.argv[1])', input], {
        cwd: fixture,
        capture: true,
      }),
      input,
    );
  });
  await t.test('missing executable fails', async () => {
    await assert.rejects(run(join(fixture, 'missing'), [], { cwd: fixture }), /ENOENT/);
  });
  await t.test('failed executable retains bounded diagnostic and exit code', async () => {
    await assert.rejects(
      run(node, ['-e', 'console.error("failure detail");process.exit(7)'], {
        cwd: fixture,
        capture: true,
      }),
      /exit 7[\s\S]*failure detail/,
    );
  });
  await t.test('timeout kills a child and does not wait for its normal completion', async () => {
    const start = Date.now();
    await assert.rejects(
      run(node, ['-e', 'setTimeout(()=>{},60000)'], { cwd: fixture, timeout: 100 }),
      /timed out/,
    );
    assert.ok(Date.now() - start < 5000);
  });
  await t.test('abort cleans up a running process', async () => {
    const controller = new AbortController();
    const promise = run(node, ['-e', 'setTimeout(()=>{},60000)'], {
      cwd: fixture,
      signal: controller.signal,
    });
    setTimeout(() => controller.abort(), 100);
    await assert.rejects(promise, /cancelled/);
  });
  await t.test('unsafe batch files are rejected without shell evaluation', async () => {
    await writeFile(join(fixture, 'vendor.cmd'), 'exit 0');
    assert.throws(
      () => run(join(fixture, 'vendor.cmd'), [], { cwd: fixture }),
      /native executable/,
    );
  });
});
