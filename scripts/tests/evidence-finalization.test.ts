import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, writeFile, rm, readdir, access } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import type { Execution } from '../evidence/types.ts';
import { persistExecution } from '../evidence/finalize.ts';
import { validateExecution } from '../evidence/validate.ts';

function execution(): Execution {
  return {
    schemaVersion: 1,
    source: { revision: 'b'.repeat(40), dirtyDiffSha256: null },
    results: ['passed', 'failed'].map((status) => ({
      testId: `node:fixture:${status}`,
      attempt: 1,
      status,
      durationMs: 1,
      timestamp: '2026-10-16T00:00:00Z',
      binarySha256: 'a'.repeat(64),
      platform: 'portable',
      toolchain: process.version,
      device: 'synthetic fixture',
      environment: { valid: true, reasons: [] },
      artifacts: [],
    })),
  };
}

await test('cancellation before or during staging publishes no certified execution', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'sketch-finalization-'));
  try {
    for (const timing of ['before', 'during', 'during-failure']) {
      const controller = new AbortController();
      const destination = join(directory, `${timing}.json`);
      if (timing === 'before') controller.abort();
      let writes = 0;
      await assert.rejects(
        persistExecution(execution(), controller.signal, destination, async (path, contents) => {
          assert.notEqual(path, destination);
          await assert.rejects(access(destination), { code: 'ENOENT' });
          await writeFile(path, contents);
          writes++;
          controller.abort();
          if (timing === 'during-failure') throw new Error('storage unavailable after write');
        }),
        timing === 'during-failure' ? /storage unavailable/ : /abort/i,
      );
      assert.equal(writes, timing === 'before' ? 0 : 1);
      await assert.rejects(access(destination), { code: 'ENOENT' });
      assert.deepEqual(await readdir(directory), []);
    }
  } finally {
    await rm(directory, { recursive: true });
  }
});

await test('atomic publication preserves outcomes and refuses to replace an earlier attempt', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'sketch-finalization-'));
  try {
    const controller = new AbortController();
    const destination = join(directory, 'execution.json');
    const original = execution();
    await persistExecution(original, controller.signal, destination);
    const recorded: Execution = JSON.parse(await readFile(destination, 'utf8'));
    validateExecution(recorded);
    assert.deepEqual(recorded, original);
    const changed = execution();
    changed.results[0].status = 'failed';
    await assert.rejects(persistExecution(changed, controller.signal, destination), {
      code: 'EEXIST',
    });
    assert.deepEqual(JSON.parse(await readFile(destination, 'utf8')), original);
    assert.deepEqual(await readdir(directory), ['execution.json']);
  } finally {
    await rm(directory, { recursive: true });
  }
});
