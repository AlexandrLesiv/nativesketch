import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { JSONReportSpec } from 'playwright/types/testReporter';
import { browserSpecs, browserOutcome } from './results.ts';
import { browserOptions } from './options.ts';
import { spec } from './spec-fixture.ts';
function report(specs: JSONReportSpec[]) {
  return {
    errors: [],
    suites: [{ title: 'editing', file: 'editing.spec.ts', line: 1, column: 1, specs }],
  };
}
await test('enumeration refuses empty, duplicate and expected-failure test sets', () => {
  assert.throws(() => browserSpecs(report([])), /Empty/);
  assert.throws(() => browserSpecs(report([spec(), spec()])), /duplicate/);
  const expectedFailure = spec();
  expectedFailure.tests[0].expectedStatus = 'failed';
  assert.throws(() => browserSpecs(report([expectedFailure])), /expected-failure/);
  assert.equal(browserSpecs(report([spec()])).length, 1);
});
await test('launch failure cannot be reported as functional acceptance', () => {
  const item = spec();
  item.tests[0].results[0].status = 'failed';
  item.tests[0].results[0].error = { message: 'browserType.launch: failed to launch' };
  assert.equal(browserOutcome(item).status, 'blocked');
  assert.equal(browserOutcome(item).environment.valid, false);
});
await test('custom browser IDs retain duplicate and project validation without changing web IDs', () => {
  const item = spec();
  item.title = 'DX-NAV-001 project navigation';
  assert.throws(() => browserSpecs(report([item])), /stable acceptance ID/);
  const identify = (value: JSONReportSpec) => `workbench:${value.title}`;
  assert.equal(browserSpecs(report([item]), identify).length, 1);
  assert.throws(() => browserSpecs(report([item, item]), identify), /duplicate/);
  item.tests[0].projectName = 'unverified-browser';
  assert.throws(() => browserSpecs(report([item]), identify), /Unexpected browser project/);
});
await test('missing status, empty result and hidden retries fail closed', () => {
  const item = spec();
  item.tests[0].results[0].status = undefined;
  assert.throws(() => browserOutcome(item), /status/);
  item.tests[0].results[0].retry = 1;
  assert.throws(() => browserOutcome(item), /retry/);
  item.tests[0].results = [];
  assert.throws(() => browserOutcome(item), /Missing/);
});
await test('assertion failure remains failed and skipped never becomes passed', () => {
  const item = spec();
  item.tests[0].results[0].status = 'failed';
  assert.equal(browserOutcome(item).status, 'failed');
  assert.equal(browserOutcome(item).environment.valid, true);
  item.tests[0].results[0].status = 'skipped';
  assert.equal(browserOutcome(item).status, 'skipped');
});
await test('output must be a fresh artifact subdirectory; headed mode and invalid attempts rejected', () => {
  for (const args of [
    ['--output', '../outside'],
    ['--output', 'artifacts'],
    ['--headed', 'true'],
    ['--output', 'artifacts/run', '--attempt', '0'],
  ])
    assert.throws(() => browserOptions(args, '/project'));
  assert.deepEqual(browserOptions(['--output', 'artifacts/run', '--attempt', '2'], '/project'), {
    output: '/project/artifacts/run',
    attempt: 2,
  });
});
