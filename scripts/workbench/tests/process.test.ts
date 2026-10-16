import test from 'node:test';
import assert from 'node:assert/strict';
import { runOwned } from '../process.ts';
await test('owned workbench child preserves failure and terminates output floods', async () => {
  const result = await runOwned(
    process.execPath,
    ['-e', 'console.log("failed");process.exit(9)'],
    process.cwd(),
    new AbortController().signal,
    () => {},
  );
  assert.equal(result.exitCode, 9);
  assert.match(result.output, /failed/);
  const flood = await runOwned(
    process.execPath,
    ['-e', 'process.stdout.write(Buffer.alloc(2000000,255));setInterval(()=>{},1000)'],
    process.cwd(),
    new AbortController().signal,
    () => {},
    5000,
  );
  assert.equal(flood.reason, 'Output exceeded 1 MiB');
  assert.ok(Buffer.byteLength(flood.output) <= 1024 * 1024);
});
await test('owned workbench cancellation waits for child cleanup', async () => {
  const controller = new AbortController();
  const result = await runOwned(
    process.execPath,
    ['-e', 'process.on("SIGTERM",()=>{});console.log("ready");setInterval(()=>{},1000)'],
    process.cwd(),
    controller.signal,
    (log) => {
      if (log.includes('ready')) controller.abort();
    },
    5000,
  );
  assert.equal(result.reason, 'Cancelled');
  assert.notEqual(result.exitCode, 0);
  assert.equal(result.signal, 'SIGKILL');
});

await test('owned workbench timeout is not reported as success', async () => {
  const result = await runOwned(
    process.execPath,
    ['-e', 'setInterval(()=>{},1000)'],
    process.cwd(),
    new AbortController().signal,
    () => {},
    50,
  );
  assert.equal(result.reason, 'Timed out');
  assert.notEqual(result.exitCode, 0);
});
