import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { freshOutput, recorded } from './evidence.ts';
import {
  requireVersion,
  validateAdvisories,
  validateTestOutput,
  requireUnchanged,
} from './validation.ts';

await test('reliability rejects mismatched tools and missing or incomplete lane output', () => {
  assert.doesNotThrow(() => requireVersion('cargo-deny 0.20.2', '0.20.2'));
  assert.throws(() => requireVersion('cargo-deny 0.20.20', '0.20.2'));
  const text =
    'test sample::contract ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out';
  assert.deepEqual(validateTestOutput(text, 1), ['sample::contract']);
  for (const invalid of [
    '',
    text.replace('0 ignored', '1 ignored'),
    text.replace('sample::contract', ''),
    text + text,
  ])
    assert.throws(() => validateTestOutput(invalid, 1));
  for (const invalid of [
    '',
    '{}',
    '{"type":"summary","fields":{"advisories":{"errors":1,"warnings":0}}}',
  ])
    assert.throws(() => validateAdvisories(invalid));
  assert.doesNotThrow(() =>
    validateAdvisories('{"type":"summary","fields":{"advisories":{"errors":0,"warnings":0}}}'),
  );
});
await test('reliability refuses stale source and preserves immutable prior output', async () => {
  const path = await mkdtemp(join(tmpdir(), 'sketch-reliability-'));
  try {
    assert.throws(() =>
      requireUnchanged({ revision: 'a', dirty: null }, { revision: 'a', dirty: 'changed' }),
    );
    await assert.rejects(freshOutput(path), /EEXIST/);
    assert.doesNotThrow(() => requireUnchanged({ revision: 'a' }, { revision: 'a' }));
  } finally {
    await rm(path, { recursive: true });
  }
});
await test('reliability retains missing-tool, failed-command and timeout evidence', async () => {
  const path = await mkdtemp(join(tmpdir(), 'sketch-reliability-failure-'));
  try {
    await assert.rejects(recorded(path, 'missing', join(path, 'absent'), [], { cwd: path }));
    await assert.rejects(
      recorded(
        path,
        'failed',
        process.execPath,
        ['-e', 'console.error("retained diagnostic");process.exit(7)'],
        { cwd: path },
      ),
    );
    await assert.rejects(
      recorded(path, 'timeout', process.execPath, ['-e', 'setTimeout(()=>{},60000)'], {
        cwd: path,
        timeout: 30,
      }),
    );
    assert.match(await readFile(join(path, 'failed.log'), 'utf8'), /retained diagnostic/);
    assert.match(await readFile(join(path, 'failed.failure.json'), 'utf8'), /"exitCode": 7/);
    assert.match(await readFile(join(path, 'timeout.failure.json'), 'utf8'), /timed out/);
    assert.match(await readFile(join(path, 'missing.failure.json'), 'utf8'), /ENOENT/);
  } finally {
    await rm(path, { recursive: true });
  }
});
