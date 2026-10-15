import test from 'node:test';
import assert from 'node:assert/strict';
import { run } from '../lib/process.mjs';
import { root } from '../lib/context.mjs';
await test('captured output stays bounded even while child ignores termination', async () => {
  const flood =
    "process.on('SIGTERM',()=>{});process.stdout.write('x'.repeat(16*1024*1024));setInterval(()=>{},1000)";
  await assert.rejects(
    run(process.execPath, ['-e', flood], { cwd: root, capture: true }),
    (error) => {
      assert.ok(error instanceof Error);
      assert.match(error.message, /output exceeded limit/);
      assert.ok(error.message.length <= 4_000_200, `Captured ${error.message.length} characters`);
      return true;
    },
  );
});
