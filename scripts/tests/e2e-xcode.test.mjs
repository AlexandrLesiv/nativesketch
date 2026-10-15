import test from 'node:test';
import assert from 'node:assert/strict';
import { enumeratedTests, executionOutcome, testOutcomes } from '../e2e/xcode-results.mjs';

const id = 'xctest:NativeSketchUITests:EditorJourneyTests/testDrawSaveReopen';
test('Xcode exit zero with enumeration errors cannot invent executable coverage', () => {
  assert.throws(
    () =>
      enumeratedTests({
        errors: ['Cannot authenticate off console'],
        values: [{ enabledTests: [{ identifier: 'NativeSketchUITests' }] }],
      }),
    /blocked/,
  );
  assert.throws(
    () => enumeratedTests({ values: [{ enabledTests: [{ identifier: 'NativeSketchUITests' }] }] }),
    /No unique/,
  );
});
test('executable discovery normalizes target class method and rejects duplicates', () => {
  const entry = { identifier: 'NativeSketchUITests/EditorJourneyTests/testDrawSaveReopen()' };
  assert.deepEqual(enumeratedTests({ values: [{ enabledTests: [entry] }] }), [id]);
  assert.throws(() => enumeratedTests({ values: [{ enabledTests: [entry, entry] }] }), /No unique/);
});
test('system initialization failures do not masquerade as test execution', () => {
  assert.deepEqual(
    testOutcomes(
      {
        testNodes: [
          {
            nodeType: 'Test Case',
            nodeIdentifier: 'Runner encountered an error',
            result: 'Failed',
          },
        ],
      },
      [id],
    ),
    [],
  );
});
test('XCTest assertions preserve failed and skipped outcomes', () => {
  const node = {
    nodeType: 'Test Case',
    nodeIdentifier: 'EditorJourneyTests/testDrawSaveReopen()',
    result: 'Failed',
    durationInSeconds: 2,
  };
  assert.deepEqual(testOutcomes({ testNodes: [{ children: [node] }] }, [id]), [
    { testId: id, status: 'failed', durationMs: 2000 },
  ]);
  assert.equal(
    testOutcomes({ testNodes: [{ ...node, result: 'Skipped' }] }, [id])[0].status,
    'skipped',
  );
});

test('runner failure invalidates a Passed case even when only result JSON is imported', () => {
  const result = executionOutcome({ status: 'passed' }, new Error('finalization failed'));
  assert.equal(result.status, 'blocked');
  assert.equal(result.environment.valid, false);
  assert.ok(result.environment.reasons.length);
  assert.equal(executionOutcome({ status: 'passed' }, undefined).environment.valid, true);
});
