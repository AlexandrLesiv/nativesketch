import test from 'node:test';
import assert from 'node:assert/strict';
import type { Discovery, Execution, Feature } from '../evidence/types.ts';
import { buildReport } from '../evidence/report.ts';
import { cargoTestId } from '../evidence/cargo-platform.ts';
const source = { revision: 'a'.repeat(40), dirtyDiffSha256: null };
const id = 'xctest:NativeSketchUITests:EditorJourneyTests/testDrawSaveReopen';
const hash = 'b'.repeat(64);
const scope = { runner: 'xctest', platform: 'macos', level: 'native-e2e' };
function fixture(): { features: Feature[]; discovery: Discovery; execution: Execution } {
  return {
    features: [
      {
        schemaVersion: 1,
        id: 'native',
        description: 'Native',
        sourceRevision: source.revision,
        cases: [
          {
            id: 'BRUSH-001',
            expected: 'Change brush and draw',
            invariants: ['INK-003'],
            ownerTask: '017',
            automation: 'planned',
            testIds: [],
            gaps: [{ reason: 'Other hosts need automation', ownerTask: '017' }],
            requirements: [
              {
                platform: 'macos',
                level: 'native-e2e',
                coverage: {
                  automation: 'implemented',
                  testIds: [id],
                  gaps: [],
                },
              },
              { platform: 'windows', level: 'native-e2e' },
            ],
          },
        ],
      },
    ],
    discovery: {
      schemaVersion: 1,
      source,
      tests: [{ id, ...scope, binarySha256: hash, testBinarySha256: hash }],
    },
    execution: {
      schemaVersion: 1,
      source,
      results: [
        {
          testId: id,
          attempt: 1,
          status: 'passed',
          binarySha256: hash,
          testBinarySha256: hash,
          platform: 'macos',
          durationMs: 1,
          timestamp: '2026-10-15T00:00:00Z',
          toolchain: 'Xcode',
          device: 'test Mac',
          environment: { valid: true, reasons: [] },
          artifacts: [],
        },
      ],
    },
  };
}
await test('requirement automation does not certify other platforms or hide gaps', () => {
  const f = fixture();
  const report = buildReport(f.features, f.discovery, [f.execution]);
  assert.deepEqual(
    report.rows.map((r) => [r.automation, r.status]),
    [
      ['implemented', 'passed'],
      ['planned', 'missing'],
    ],
  );
  assert.equal(report.passed, false);
  assert.deepEqual(report.issues, []);
  f.features[0].cases[0].requirements[0].coverage?.gaps.push({
    reason: 'Missing oracle',
    ownerTask: '017',
  });
  assert.equal(buildReport(f.features, f.discovery, [f.execution]).rows[0].status, 'gap');
});
await test('background host absence remains missing discovery, never a pass', () => {
  const f = fixture();
  f.discovery.tests = [];
  const report = buildReport(f.features, f.discovery, []);
  assert.equal(report.rows[0].status, 'missing-discovery');
  assert.deepEqual(report.rows[0].testIds, [id]);
  assert.deepEqual(report.rows[0].discoveredTestIds, []);
  assert.deepEqual(report.issues, []);
  assert.equal(report.passed, false);
});
await test('enumerated native scope rejects typo and dangling ID even with zero discovered tests', () => {
  for (const explicit of [false, true]) {
    const f = fixture();
    if (explicit) {
      f.discovery.tests = [];
      f.discovery.scopes = [scope];
    } else f.discovery.tests[0].id += 'Different';
    const report = buildReport(f.features, f.discovery, []);
    assert.match(report.issues[0], /Undiscovered required test/);
    assert.equal(report.rows[0].status, 'invalid-mapping');
    assert.equal(report.passed, false);
  }
});
await test('native mappings reject wrong platform, level and pointer width', () => {
  for (const change of [{ platform: 'windows' }, { level: 'unit' }, { pointerWidth: 32 }]) {
    const f = fixture();
    f.features[0].cases[0].requirements[0].pointerWidth = 64;
    Object.assign(f.discovery.tests[0], { pointerWidth: 64 }, change);
    const report = buildReport(f.features, f.discovery, []);
    assert.match(report.issues[0], /Orphan/);
    assert.equal(report.rows[0].status, 'invalid-mapping');
  }
});
await test('host-qualified Cargo IDs cannot certify or invalidate an unenumerated native host', () => {
  for (const platform of ['windows', 'linux']) {
    const f = fixture();
    const expectedId = cargoTestId('services', 'macos', 'store', 'write');
    const foreignId = cargoTestId('services', platform, 'store', 'write');
    const requirement = f.features[0].cases[0].requirements[0];
    assert.ok(requirement.coverage);
    requirement.coverage.testIds = [expectedId];
    requirement.level = 'integration';
    Object.assign(f.discovery.tests[0], { id: foreignId, runner: 'cargo', level: 'integration' });
    f.execution.results[0].testId = foreignId;
    f.discovery.tests[0].platform = platform;
    f.execution.results[0].platform = platform;
    let report = buildReport(f.features, f.discovery, [f.execution]);
    assert.deepEqual(report.issues, []);
    assert.equal(report.rows[0].status, 'missing-discovery');
    assert.deepEqual(report.rows[0].discoveredTestIds, []);
    assert.deepEqual(report.rows[0].attempts, []);
    assert.equal(report.passed, false);
    // If the requested host was enumerated, a copy on another OS cannot hide its missing ID.
    f.discovery.scopes = [{ runner: 'cargo', platform: 'macos', level: 'integration' }];
    report = buildReport(f.features, f.discovery, [f.execution]);
    assert.match(report.issues[0], /Undiscovered required test/);
    assert.equal(report.rows[0].status, 'invalid-mapping');
  }
});
await test('native requirement mappings retain source, binary and retry safeguards', () => {
  const f = fixture();
  f.execution.source = { ...source, revision: 'c'.repeat(40) };
  assert.equal(buildReport(f.features, f.discovery, [f.execution]).rows[0].status, 'stale');
  f.execution.source = source;
  f.execution.results[0].testBinarySha256 = 'd'.repeat(64);
  assert.equal(buildReport(f.features, f.discovery, [f.execution]).rows[0].status, 'mismatched');
  f.execution.results[0].testBinarySha256 = hash;
  f.execution.results.push({ ...f.execution.results[0], attempt: 2, status: 'failed' });
  assert.equal(buildReport(f.features, f.discovery, [f.execution]).rows[0].status, 'flaky');
});
await test('partial override, malformed namespaces and duplicate scopes are rejected', () => {
  const f = fixture();
  const coverage = f.features[0].cases[0].requirements[0].coverage;
  assert.ok(coverage);
  coverage.testIds = ['unqualified'];
  assert.throws(() => buildReport(f.features, f.discovery, []), /namespace/);
  coverage.testIds = [id];
  f.discovery.scopes = [
    scope,
    { level: scope.level, platform: scope.platform, runner: scope.runner },
  ];
  assert.throws(() => buildReport(f.features, f.discovery, []), /Duplicate/);
  f.discovery.scopes = [];
  const incomplete: unknown = { automation: 'implemented' };
  Object.assign(f.features[0].cases[0].requirements[0], { coverage: incomplete });
  assert.throws(() => buildReport(f.features, f.discovery, []), /missing testIds/);
});
