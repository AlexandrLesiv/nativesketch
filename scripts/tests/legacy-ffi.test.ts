import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { command } from '../lib/context.ts';
import { checkLegacyFfi } from '../platforms/legacy-ffi.ts';
import { validateDiscovery, validateExecution } from '../evidence/validate.ts';

await test(
  'legacy compiler setup failure cannot claim enumeration or execution',
  {
    skip: process.platform !== 'darwin' ? 'macOS-only compiler runner' : false,
  },
  async () => {
    const output = await mkdtemp(join(tmpdir(), 'legacy-ffi-failed-setup-'));
    const invoked: string[] = [];
    try {
      await assert.rejects(
        checkLegacyFfi(output, async (executable, args, options) => {
          invoked.push(executable);
          assert.equal(executable, 'clang');
          assert.deepEqual(args, ['--version']);
          // A real subprocess failure passes through the production capture/error path.
          return command(process.execPath, ['-e', 'process.exit(19)'], options);
        }),
        /exit 19,/,
      );
      assert.deepEqual(invoked, ['clang']);
      const discovery = JSON.parse(await readFile(join(output, 'discovery.json'), 'utf8'));
      const execution = JSON.parse(await readFile(join(output, 'results.json'), 'utf8'));
      validateDiscovery(discovery);
      validateExecution(execution);
      assert.deepEqual(discovery.tests, []);
      assert.deepEqual(discovery.scopes ?? [], []);
      assert.deepEqual(execution.results, []);
      assert.match(await readFile(join(output, 'failure.json'), 'utf8'), /exit 19,/);
      assert.match(await readFile(join(output, 'toolchain.log'), 'utf8'), /exit 19,/);
    } finally {
      await rm(output, { recursive: true, force: true });
    }
  },
);
