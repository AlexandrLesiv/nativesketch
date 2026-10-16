import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { executeFuzzTargets } from './fuzz-execution.ts';
import { sourceIdentity } from '../evidence/identity.ts';
import { root } from '../lib/context.ts';
import { ProcessFailure } from '../lib/process-failure.ts';

async function checkFailure(failing: string) {
  const output = await mkdtemp(join(tmpdir(), 'sketch-fuzz-results-'));
  const hash = 'a'.repeat(64);
  const targets = ['document_codec', 'command_sequence'].map((name) => ({
    name,
    binary: name,
    binarySha256: hash,
    core: { fingerprintSha256: hash, fingerprint: {} },
  }));
  try {
    await assert.rejects(
      executeFuzzTargets(
        root,
        output,
        await sourceIdentity(root),
        'fixture',
        2,
        targets,
        async (name) => {
          if (name === failing)
            throw new ProcessFailure(name, 77, null, '', 'ERROR: assertion failed');
          return 'INFO: Loaded 1 modules\n#20 DONE cov: 44\nDone 20 runs in 1 second(s)\nstat::number_of_executed_units: 20';
        },
      ),
      /77/,
    );
    const value: { results: { status: string; attempt: number }[] } = JSON.parse(
      await readFile(join(output, 'results.json'), 'utf8'),
    );
    assert.deepEqual(
      value.results.map((result) => [result.status, result.attempt]),
      targets.map(({ name }) => [name === failing ? 'failed' : 'passed', 2]),
    );
  } finally {
    await rm(output, { recursive: true });
  }
}

await test('fuzz records a failing target without turning it into a pass or dropping earlier attempts', () =>
  checkFailure('command_sequence'));

await test('fuzz continues independent targets after a child failure and fails the aggregate', () =>
  checkFailure('document_codec'));
