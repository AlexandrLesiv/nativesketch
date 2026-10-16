import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, mkdir, copyFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { executeFuzzTargets } from './fuzz-execution.ts';
import { sourceIdentity } from '../evidence/identity.ts';
import { root } from '../lib/context.ts';
import { ProcessFailure } from '../lib/process-failure.ts';
import { run } from '../lib/process.ts';

async function withFixture(check: (fixture: string, output: string) => Promise<void>) {
  const directory = await mkdtemp(join(tmpdir(), 'sketch-fuzz-results-'));
  const fixture = join(directory, 'repo'),
    output = join(directory, 'evidence');
  await mkdir(join(fixture, 'crates/sketch-core/tests/fixtures'), { recursive: true });
  await mkdir(output);
  await copyFile(
    join(root, 'crates/sketch-core/tests/fixtures/portable-v1.nsketch'),
    join(fixture, 'crates/sketch-core/tests/fixtures/portable-v1.nsketch'),
  );
  try {
    const git = (args: string[]) => run('git', args, { cwd: fixture, capture: true });
    await git(['init', '-q']);
    await git(['add', '.']);
    await git([
      '-c',
      'user.name=Fixture',
      '-c',
      'user.email=fixture@example.invalid',
      'commit',
      '-qm',
      'fixture',
    ]);
    await check(fixture, output);
  } finally {
    await rm(directory, { recursive: true });
  }
}

function fixtureTargets() {
  const hash = 'a'.repeat(64);
  return ['document_codec', 'command_sequence'].map((name) => ({
    name,
    binary: name,
    binarySha256: hash,
    core: { fingerprintSha256: hash, fingerprint: {} },
  }));
}

async function checkFailure(failing: string) {
  await withFixture(async (fixture, output) => {
    const targets = fixtureTargets();
    await assert.rejects(
      executeFuzzTargets(
        fixture,
        output,
        await sourceIdentity(fixture),
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
  });
}

await test('fuzz records a failing target without turning it into a pass or dropping earlier attempts', () =>
  checkFailure('command_sequence'));

await test('fuzz continues independent targets after a child failure and fails the aggregate', () =>
  checkFailure('document_codec'));

await test('fuzz source mutation stops before the next target and preserves invalid evidence', () =>
  withFixture(async (fixture, output) => {
    const called: string[] = [];
    await assert.rejects(
      executeFuzzTargets(
        fixture,
        output,
        await sourceIdentity(fixture),
        'fixture',
        1,
        fixtureTargets(),
        async (name) => {
          called.push(name);
          await writeFile(join(fixture, 'changed.txt'), 'changed');
          throw new ProcessFailure(name, 77, null, '', 'ERROR: assertion failed');
        },
      ),
      /Source changed/,
    );
    assert.deepEqual(called, ['document_codec']);
    const value: {
      results: { status: string; environment: { valid: boolean; reasons: string[] } }[];
    } = JSON.parse(await readFile(join(output, 'results.json'), 'utf8'));
    assert.deepEqual(
      value.results.map((r) => r.status),
      ['failed'],
    );
    assert.equal(value.results[0].environment.valid, false);
    assert.match(value.results[0].environment.reasons.join(' '), /Source changed/);
  }));
