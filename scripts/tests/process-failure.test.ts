import test from 'node:test';
import assert from 'node:assert/strict';
import { run } from '../lib/process.ts';
import { ProcessFailure } from '../lib/process-failure.ts';

await test('real command failure retains structured exit, signal and bounded output', async () => {
  await assert.rejects(
    run(process.execPath, ['-e', 'console.error("test failure");process.exit(65)'], {
      cwd: process.cwd(),
      capture: true,
    }),
    (error: unknown) => {
      assert.ok(error instanceof ProcessFailure);
      assert.equal(error.exitCode, 65);
      assert.equal(error.signal, null);
      assert.equal(error.reason, '');
      assert.match(error.output, /test failure/);
      return true;
    },
  );
});
await test('pre-cancel and timeout are distinguishable from a normal exit', async () => {
  const controller = new AbortController();
  controller.abort();
  for (const options of [{ signal: controller.signal }, { timeout: 20 }])
    await assert.rejects(
      run(process.execPath, ['-e', 'setTimeout(()=>{},60000)'], {
        cwd: process.cwd(),
        ...options,
      }),
      (error: unknown) => {
        assert.ok(error instanceof ProcessFailure);
        assert.match(error.reason, /cancelled|timed out/);
        assert.notEqual(error.exitCode, 65);
        return true;
      },
    );
});
