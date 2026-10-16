import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fixture } from './fixture.ts';
import { sourceIdentity } from '../identity.ts';
import { coverage } from '../adapters/coverage.ts';
await test('workbench aggregate producer envelopes preserve failed retries and stale attempts', async () => {
  const f = await fixture();
  try {
    const manifest = 'quality/features/fixture.json';
    const feature = JSON.parse(await readFile(join(f.root, manifest), 'utf8'));
    feature.cases[0].testIds = ['node:fixture'];
    feature.cases[0].automation = 'implemented';
    feature.cases[0].gaps = [];
    await f.put(manifest, JSON.stringify(feature));
    const source = await sourceIdentity(f.root);
    const binarySha256 = 'a'.repeat(64);
    await f.put(
      'artifacts/evidence/discovery.json',
      JSON.stringify({
        schemaVersion: 1,
        source,
        tests: [
          { id: 'node:fixture', runner: 'node', platform: 'portable', level: 'unit', binarySha256 },
        ],
      }),
    );
    const result = {
      testId: 'node:fixture',
      attempt: 1,
      status: 'failed',
      durationMs: 1,
      timestamp: new Date().toISOString(),
      binarySha256,
      platform: 'portable',
      toolchain: process.version,
      device: 'fixture',
      environment: { valid: true, reasons: [] },
      artifacts: [],
    };
    const first = { schemaVersion: 1, source, results: [result] };
    const second = {
      schemaVersion: 1,
      source,
      results: [{ ...result, attempt: 2, status: 'passed' }],
    };
    const inspect = () => coverage(f.root, [manifest], source);
    await f.put('artifacts/evidence/results.json', JSON.stringify([first, second]));
    let observed = await inspect();
    assert.equal(observed.state, 'ready');
    assert.equal(observed.data.rows[0].status, 'flaky');
    assert.equal(observed.data.rows[0].attempts.length, 2);
    await f.put(
      'artifacts/evidence/results.json',
      JSON.stringify([first, { ...second, source: { ...source, revision: 'b'.repeat(40) } }]),
    );
    observed = await inspect();
    assert.equal(observed.data.rows[0].status, 'stale');
    assert.equal(observed.data.rows[0].attempts.length, 2);
    await f.put('artifacts/evidence/results.json', JSON.stringify(first));
    assert.equal((await inspect()).data.rows[0].status, 'failed');
    for (const malformed of [[first, {}], [[first]], null]) {
      await f.put('artifacts/evidence/results.json', JSON.stringify(malformed));
      assert.equal((await inspect()).state, 'invalid');
    }
  } finally {
    await f.close();
  }
});
