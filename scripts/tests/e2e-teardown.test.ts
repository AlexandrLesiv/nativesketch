import test from 'node:test';
import assert from 'node:assert/strict';
import { validateTeardownRegression } from '../e2e/teardown-regression.ts';

const log = 'testAssertionAbortRunsCleanup failed\ntestThrowingCleanupPreservesLIFO failed';
const trace = [
  'assertion:body',
  'assertion:cleanup',
  'throwing:body',
  'throwing:inner',
  'throwing:outer',
]
  .map((event) => JSON.stringify({ event }))
  .join('\n');
await test('negative native cleanup suite requires actual failures and exact LIFO completion', () => {
  assert.doesNotThrow(() => validateTeardownRegression(1, log, trace));
  assert.throws(() => validateTeardownRegression(0, log, trace), /unexpectedly/);
  assert.throws(() => validateTeardownRegression(null, log, trace), /unexpectedly/);
  assert.throws(() => validateTeardownRegression(1, '', trace), /Missing/);
  assert.throws(
    () => validateTeardownRegression(1, log, trace.split('\n').slice(0, -1).join('\n')),
    /sequence/,
  );
  assert.throws(
    () => validateTeardownRegression(1, log, `${trace}\n{"event":"unexpected-after-failure"}`),
    /sequence/,
  );
});
