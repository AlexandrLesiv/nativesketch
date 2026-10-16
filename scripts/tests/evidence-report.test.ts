import type { Feature, Result } from '../evidence/types.ts';
import test from 'node:test';
import assert from 'node:assert/strict';
import { buildReport, renderHtml } from '../evidence/report.ts';
import { validateExecution, validateFeatures } from '../evidence/validate.ts';
const hash = 'a'.repeat(64);
const source = { revision: 'b'.repeat(40), dirtyDiffSha256: null };

function features(): Feature[] {
  return [
    {
      schemaVersion: 1,
      id: 'ink',
      description: 'Ink',
      sourceRevision: source.revision,
      cases: [
        {
          id: 'INK-001',
          expected: 'Cancel safely',
          invariants: ['INK-001'],
          ownerTask: '018',
          requirements: [{ platform: 'portable', level: 'unit' }],
          testIds: ['node:fixture:cancel'],
          automation: 'implemented',
          gaps: [],
        },
      ],
    },
  ];
}
function discovery() {
  return {
    schemaVersion: 1,
    source,
    tests: [
      {
        id: 'node:fixture:cancel',
        runner: 'node',
        platform: 'portable',
        level: 'unit',
        binarySha256: hash,
      },
    ],
  };
}

function result(status: string = 'passed', attempt: number = 1): Result {
  return {
    testId: 'node:fixture:cancel',
    attempt,
    status,
    binarySha256: hash,
    platform: 'portable',
    durationMs: 1,
    timestamp: '2026-10-15T00:00:00Z',
    toolchain: 'node24',
    device: 'synthetic fixture',
    environment: { valid: true, reasons: [] },
    artifacts: [],
  };
}

const execution = (results: Result[]) => ({ schemaVersion: 1, source, results });
await test('missing, skip, failure, unsupported and retry retain nonpassing evidence', () => {
  for (const state of ['failed', 'skipped', 'blocked', 'unsupported']) {
    const report = buildReport(features(), discovery(), [execution([result(state)])]);
    assert.equal(report.rows[0].status, state);
    assert.equal(report.passed, false);
  }
  assert.equal(buildReport(features(), discovery(), []).rows[0].status, 'missing');
  const retry = buildReport(features(), discovery(), [
    execution([result('failed'), result('passed', 2)]),
  ]);
  assert.equal(retry.rows[0].status, 'flaky');
  assert.equal(retry.rows[0].attempts.length, 2);
});
await test('exact source, binary, platform and environment are required', () => {
  const stale = execution([result()]);
  stale.source = { ...source, revision: 'c'.repeat(40) };
  assert.equal(buildReport(features(), discovery(), [stale]).rows[0].status, 'stale');
  for (const change of [{ binarySha256: 'd'.repeat(64) }, { platform: 'macos' }]) {
    const row = buildReport(features(), discovery(), [execution([{ ...result(), ...change }])])
      .rows[0];
    assert.equal(row.status, 'mismatched');
  }
  const invalid = { ...result(), environment: { valid: false, reasons: ['locked'] } };
  assert.equal(
    buildReport(features(), discovery(), [execution([invalid])]).rows[0].status,
    'invalid-environment',
  );
});
await test('known passing automation cannot satisfy native or incomplete acceptance', () => {
  const registry = features();
  registry[0].cases[0].requirements = [{ platform: 'macos', level: 'native-e2e' }];
  const report = buildReport(registry, discovery(), [execution([result()])]);
  assert.equal(report.passed, false);
  assert.match(report.issues[0], /Orphan/);
  const partial = features();
  partial[0].cases[0].gaps.push({ reason: 'Missing case', ownerTask: '018' });
  assert.equal(buildReport(partial, discovery(), [execution([result()])]).rows[0].status, 'gap');
  assert.equal(buildReport(features(), discovery(), [execution([result()])]).passed, true);
});
await test('duplicates, unknown IDs and malformed evidence fail validation', () => {
  assert.throws(
    () => buildReport(features(), discovery(), [execution([result(), result()])]),
    /Duplicate/,
  );
  assert.throws(
    () => buildReport(features(), discovery(), [execution([{ ...result(), testId: 'unknown' }])]),
    /Unknown/,
  );
  assert.throws(() => validateFeatures([...features(), ...features()]), /Duplicate/);
  assert.throws(() => validateExecution(execution([{ ...result(), attempt: 0 }])), /attempt/);
  for (const artifact of ['../secret', '/secret', 'javascript:alert(1)', 'C:\\secret']) {
    assert.throws(
      () => validateExecution(execution([{ ...result(), artifacts: [artifact] }])),
      /Unsafe/,
    );
  }
});
await test('HTML escapes feature strings and artifact attributes', () => {
  const registry = features();
  registry[0].cases[0].expected = '<script>alert("x")</script>';
  const report = buildReport(registry, discovery(), [
    execution([{ ...result(), artifacts: ['proof" onclick="bad.png'] }]),
  ]);
  const html = renderHtml(report);
  assert.ok(!html.includes('<script>'));
  assert.ok(html.includes('&lt;script&gt;'));
  assert.ok(html.includes('href="proof%22%20onclick%3D%22bad.png"'));
});
