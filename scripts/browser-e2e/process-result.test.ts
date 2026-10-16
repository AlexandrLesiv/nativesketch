import test from 'node:test';
import assert from 'node:assert/strict';
import { run } from '../lib/process.ts';
import { ProcessFailure } from '../lib/process-failure.ts';
import {
  failedProcess,
  executionReasons,
  certifiedOutcome,
  processSucceeded,
} from './process-result.ts';
import { spec } from './spec-fixture.ts';

await test('a valid pass report followed by actual runner timeout cannot certify acceptance', async () => {
  let failure: unknown;
  try {
    await run(
      process.execPath,
      ['-e', 'console.log("report-written"); setInterval(() => {}, 1000)'],
      {
        cwd: process.cwd(),
        capture: true,
        timeout: 500,
      },
    );
  } catch (error) {
    failure = error;
  }
  assert.ok(failure instanceof ProcessFailure);
  assert.match(failure.output, /report-written/);
  const completion = failedProcess(failure);
  assert.equal(processSucceeded(completion), false);
  const result = certifiedOutcome(spec(), executionReasons(completion, [spec()]));
  assert.equal(result.status, 'blocked');
  assert.equal(result.environment.valid, false);
  assert.match(result.environment.reasons.join(), /timed out/);
});

await test('cancel signal overflow spawn and unexplained exits invalidate completed reports', () => {
  for (const error of [
    new ProcessFailure('runner', 0, null, 'Process cancelled', ''),
    new ProcessFailure('runner', 1, 'SIGTERM', '', ''),
    new ProcessFailure('runner', 1, null, 'Process output exceeded limit', ''),
    new ProcessFailure('runner', null, null, 'Process spawn failed', ''),
    new ProcessFailure('runner', 2, null, '', ''),
    new ProcessFailure('runner', 1, null, '', ''),
    new Error('unknown execution failure'),
  ]) {
    const completion = failedProcess(error);
    assert.equal(processSucceeded(completion), false);
    const result = certifiedOutcome(spec(), executionReasons(completion, [spec()]));
    assert.equal(result.status, 'blocked');
    assert.equal(result.environment.valid, false);
  }
});

await test('normal attributable assertion failure preserves independently passed peers', () => {
  const failed = spec();
  failed.tests[0].results[0].status = 'failed';
  const completion = failedProcess(new ProcessFailure('runner', 1, null, '', 'assertion failed'));
  const reasons = executionReasons(completion, [spec(), failed]);
  assert.deepEqual(reasons, []);
  assert.equal(certifiedOutcome(spec(), reasons).status, 'passed');
  assert.equal(certifiedOutcome(failed, reasons).status, 'failed');
  assert.equal(certifiedOutcome(failed, reasons).environment.valid, true);
  assert.equal(processSucceeded(completion), false);
  assert.ok(executionReasons({ exitCode: 0, signal: null, reason: '' }, [failed]).length);
});
