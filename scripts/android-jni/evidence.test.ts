import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { exclusiveDirectory, assertStable, captureStage, publishQualification } from './session.ts';
import { harvest, normalized } from './evidence.ts';
test('failed Gradle after case XML retains raw and normalized failure with invalid qualification', async () => {
  const root = await mkdtemp(join(tmpdir(), 'jni-evidence-'));
  try {
    const output = join(root, 'attempt');
    await exclusiveDirectory(output);
    const results = join(root, 'apps/android/app/build/test-results/portableJniHostTest');
    await mkdir(results, { recursive: true });
    await assert.rejects(
      captureStage(output, 'gradle', async () => {
        await writeFile(
          join(results, 'TEST-fixture.xml'),
          '<testsuite tests="1" failures="1" errors="0" skipped="0" timestamp="2026-10-16T00:00:00"><testcase name="actualCase" classname="dev.nativesketch.portable.InputJniHostTest" time="0.02"><failure message="fixture failure"/></testcase></testsuite>',
        );
        throw new Error('Gradle failed after writing XML');
      }),
    );
    const evidence = await harvest(root, output, [
      'dev.nativesketch.portable.InputJniHostTest:actualCase',
    ]);
    assert.ok(evidence.issues.length > 0);
    assert.equal(evidence.lanes[0]?.rows[0]?.status, 'failed');
    assert.match(
      await readFile(join(output, 'portableJniHostTest/TEST-fixture.xml'), 'utf8'),
      /fixture failure/,
    );
    await normalized(
      output,
      evidence,
      { revision: 'a'.repeat(40), dirtyDiffSha256: null },
      { production: 'b'.repeat(64), probe: 'c'.repeat(64), classes: 'd'.repeat(64) },
      'fixture-jvm',
      ['Gradle failed'],
    );
    const execution = JSON.parse(await readFile(join(output, 'execution.json'), 'utf8')) as {
      results: { status: string; environment: { valid: boolean } }[];
    };
    assert.equal(execution.results[0]?.status, 'failed');
    assert.equal(execution.results[0]?.environment.valid, false);
    await assert.rejects(exclusiveDirectory(output));
    assert.match(await readFile(join(output, 'gradle.log'), 'utf8'), /Gradle failed/);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
test('source or binary drift fails qualification while identical identities are accepted', () => {
  assertStable({ source: 'one', binary: 'a' }, { source: 'one', binary: 'a' });
  assert.throws(() => assertStable({ source: 'one' }, { source: 'two' }));
  assert.throws(() => assertStable({ binary: 'a' }, { binary: 'b' }));
});
test('late cancellation during staged qualification cannot publish passing normalized evidence', async () => {
  const output = await mkdtemp(join(tmpdir(), 'jni-abort-'));
  try {
    const controller = new AbortController();
    const issues: string[] = [];
    let builds = 0;
    await publishQualification(output, controller.signal, issues, () => {
      builds++;
      if (builds === 1) queueMicrotask(() => controller.abort());
      return {
        'identity.json': { passed: issues.length === 0 },
        'execution.json': { results: [{ environment: { valid: issues.length === 0 } }] },
      };
    });
    assert.equal(builds, 2);
    assert.match(issues.join(), /cancelled/);
    assert.equal(
      JSON.parse(await readFile(join(output, 'failed-execution.json'), 'utf8')).results[0]
        .environment.valid,
      false,
    );
    assert.equal(
      JSON.parse(await readFile(join(output, 'failed-execution.json'), 'utf8')).results[0]
        .environment.valid,
      false,
    );
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});
test('metadata collision prevents execution certification', async () => {
  const output = await mkdtemp(join(tmpdir(), 'jni-collision-'));
  try {
    await writeFile(join(output, 'identity.json'), 'retained prior evidence');
    await assert.rejects(
      publishQualification(output, new AbortController().signal, [], () => ({
        'identity.json': { checksPassed: true },
        'discovery.json': { tests: [] },
        'execution.json': { results: [{ environment: { valid: true } }] },
      })),
    );
    await assert.rejects(readFile(join(output, 'execution.json')));
    assert.equal(await readFile(join(output, 'identity.json'), 'utf8'), 'retained prior evidence');
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});
