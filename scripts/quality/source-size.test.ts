import assert from 'node:assert/strict';
import { test } from 'node:test';
import { physicalLines, checkSourceSize } from './source-size.ts';

const policy = {
  maximumPhysicalLines: 200,
  sourceExtensions: ['.rs', '.ts', '.html'],
  excludedSources: [],
};
const content = (lines: number) => '// handwritten\n'.repeat(lines);
const baseline = [
  { path: 'old.rs', category: 'handwritten-source', physicalLines: 210, extractionTask: '018' },
];
const tasks = new Set(['018']);

await test('counts physical blank/comment lines and an unterminated final line', () => {
  assert.equal(physicalLines(''), 0);
  assert.equal(physicalLines('\n'), 1);
  assert.equal(physicalLines('// comment\n\nlast'), 3);
  assert.equal(physicalLines('one\r\ntwo\r\n'), 2);
});

await test('new 201-line source fails, including handwritten HTML and extensionless scripts', () => {
  for (const path of ['new.rs', 'page.html', 'scripts/check']) {
    const text = path === 'scripts/check' ? '#!/usr/bin/env node\n' + content(200) : content(201);
    assert.equal(checkSourceSize([{ path, text }], baseline, policy, tasks).failures.length, 1);
  }
  assert.equal(
    checkSourceSize([{ path: 'new.ts', text: content(200) }], baseline, policy, tasks).failures
      .length,
    0,
  );
});

await test('legacy source may shrink, cannot grow, and final acceptance rejects all debt', () => {
  const unchanged = [{ path: 'old.rs', text: content(210) }];
  assert.equal(checkSourceSize(unchanged, baseline, policy, tasks).debt.length, 1);
  assert.equal(checkSourceSize(unchanged, baseline, policy, tasks, true).failures.length, 1);
  assert.equal(
    checkSourceSize([{ path: 'old.rs', text: content(211) }], baseline, policy, tasks).failures
      .length,
    1,
  );
  assert.equal(
    checkSourceSize([{ path: 'old.rs', text: content(200) }], baseline, policy, tasks).debt.length,
    0,
  );
});

await test('legacy allowance requires an actual task and handwritten baseline classification', () => {
  const files = [{ path: 'old.rs', text: content(210) }];
  assert.equal(checkSourceSize(files, baseline, policy, new Set()).failures.length, 1);
  const misclassified = [{ ...baseline[0], category: 'fixture-data' }];
  assert.equal(checkSourceSize(files, misclassified, policy, tasks).failures.length, 1);
});
