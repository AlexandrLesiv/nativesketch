import test from 'node:test';
import assert from 'node:assert/strict';
import { writeFileSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { allocateOwnership, register, ownershipVariable } from '../lib/owned-registry.ts';
import { runOwned } from '../workbench/process.ts';
await test(
  'owned registry missing malformed and cancelling parents fail before payload start',
  { skip: process.platform === 'win32' },
  async () => {
    const prior = process.env[ownershipVariable];
    delete process.env[ownershipVariable];
    const { owner } = allocateOwnership();
    // Only identity validation is exercised; no signals are sent to this synthetic entry.
    register(owner, null, process.pid);
    try {
      for (const value of [
        '{}',
        JSON.stringify({ ...owner, token: 'a'.repeat(48) }),
        JSON.stringify({ ...owner, scope: 'b'.repeat(36) }),
      ]) {
        process.env[ownershipVariable] = value;
        await assert.rejects(
          runOwned(
            process.execPath,
            ['-e', 'process.exit(0)'],
            process.cwd(),
            new AbortController().signal,
            () => {},
          ),
          /ownership/i,
        );
      }
      writeFileSync(join(owner.directory, `${owner.scope}.cancel`), '');
      process.env[ownershipVariable] = JSON.stringify(owner);
      await assert.rejects(
        runOwned(
          process.execPath,
          ['-e', 'process.exit(0)'],
          process.cwd(),
          new AbortController().signal,
          () => {},
        ),
        /cancelling/,
      );
    } finally {
      if (prior === undefined) delete process.env[ownershipVariable];
      else process.env[ownershipVariable] = prior;
      rmSync(owner.directory, { recursive: true });
    }
  },
);
