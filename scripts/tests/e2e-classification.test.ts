import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { completedTestFailures } from '../e2e/execution-classification.ts';
import { ProcessFailure } from '../lib/process-failure.ts';
import { testOutcomes, executionOutcome } from '../e2e/xcode-results.ts';
import { record } from '../e2e/xcresult-values.ts';
import { testIdentifier } from '../e2e/test-identifiers.ts';

function fixture(name: string): unknown {
  return JSON.parse(
    readFileSync(new URL(`fixtures/xcresult-${name}.json`, import.meta.url), 'utf8'),
  );
}
function setup() {
  const summary = record(fixture('summary'));
  const tree = record(fixture('tests'));
  const ids: string[] = [];
  const visit = (value: unknown) => {
    const node = record(value);
    if (node.nodeType === 'Test Case')
      ids.push(`xctest:NativeSketchUITests:${String(node.nodeIdentifier).replace(/\(\)$/, '')}`);
    if (Array.isArray(node.children)) node.children.forEach(visit);
  };
  if (!Array.isArray(tree.testNodes)) throw new Error('Invalid fixture');
  tree.testNodes.forEach(visit);
  const invocation = record(fixture('invocation'));
  const actions = record(invocation.actions)._values;
  if (!Array.isArray(actions)) throw new Error('Invalid fixture');
  const action = record(actions[0]);
  const outcomes = testOutcomes(tree, ids);
  const error = new ProcessFailure('xcodebuild', 65, null, '', 'retained raw output');
  return { summary, invocation, action, outcomes, ids, error };
}
function classify(f: ReturnType<typeof setup>, error: unknown = f.error) {
  return completedTestFailures(error, f.invocation, f.summary, f.outcomes, f.ids);
}
await test('real sanitized 11-case run preserves seven certified peers despite four failures', () => {
  const f = setup();
  assert.equal(classify(f).kind, 'completed-test-failures');
  assert.equal(f.outcomes.filter((item) => item.status === 'passed').length, 7);
  assert.equal(f.outcomes.filter((item) => item.status === 'failed').length, 4);
  assert.equal(f.error.exitCode, 65);
  assert.equal(f.error.output, 'retained raw output');
  assert.equal(
    executionOutcome({ status: 'passed' }, new Error('provenance changed')).status,
    'blocked',
  );
});
await test('non-normal exits cannot certify passing peers', () => {
  for (const error of [
    new Error('exit 65'),
    new ProcessFailure('xcodebuild', 0, null, '', ''),
    new ProcessFailure('xcodebuild', 65, 'SIGTERM', '', ''),
    ...[
      'Process timed out',
      'Process cancelled',
      'Process output exceeded limit',
      'spawn failed',
    ].map((reason) => new ProcessFailure('xcodebuild', 65, null, reason, '')),
  ])
    assert.throws(() => classify(setup(), error));
});
await test('incomplete selection, duplicated outcomes and summary disagreement fail closed', () => {
  for (const change of [
    (f: ReturnType<typeof setup>) => {
      f.outcomes.pop();
    },
    (f: ReturnType<typeof setup>) => {
      f.outcomes.push(f.outcomes[0]);
    },
    (f: ReturnType<typeof setup>) => {
      f.summary.passedTests = 8;
    },
    (f: ReturnType<typeof setup>) => {
      f.summary.expectedFailures = 1;
    },
    (f: ReturnType<typeof setup>) => {
      f.summary.finishTime = 0;
    },
    (f: ReturnType<typeof setup>) => {
      delete f.invocation.metrics;
    },
    (f: ReturnType<typeof setup>) => {
      delete f.action.endedTime;
    },
    (f: ReturnType<typeof setup>) => {
      record(f.action.actionResult).status = { _type: { _name: 'String' }, _value: 'cancelled' };
    },
  ]) {
    const f = setup();
    change(f);
    assert.throws(() => classify(f));
  }
});
await test('runner finalization errors cannot hide behind complete test counts', () => {
  const f = setup();
  record(f.invocation.issues).errorSummaries = { _type: { _name: 'Array' }, _values: [{}] };
  assert.throws(() => classify(f));
  const other = setup();
  record(record(other.action.actionResult).metrics).errorCount = {
    _type: { _name: 'Int' },
    _value: '1',
  };
  assert.throws(() => classify(other));
});
await test('attributed teardown issues remain failed; duplicated issue levels reconcile', () => {
  const f = setup();
  for (const container of [f.invocation, record(f.action.actionResult)]) {
    const issues = record(record(container.issues).testFailureSummaries)._values;
    if (!Array.isArray(issues)) throw new Error('Invalid fixture');
    const extra = structuredClone(record(issues[0]));
    extra.message = { _type: { _name: 'String' }, _value: 'Cleanup threw an error' };
    issues.push(extra);
  }
  assert.equal(classify(f).kind, 'completed-test-failures');
  assert.equal(
    f.outcomes.find((item) => item.testId.includes('testDrawSaveReopen'))?.status,
    'failed',
  );
  record(record(f.invocation.issues).testFailureSummaries)._values = [];
  assert.throws(() => classify(f));
});
await test('aliases are exact and ambiguous targets are rejected', () => {
  const ids = ['xctest:A:Module.Class/testExample', 'xctest:B:Module.Class/testExample'];
  assert.throws(() => testIdentifier('Module.Class.testExample()', ids), /Ambiguous/);
  assert.equal(testIdentifier('A/Module.Class/testExample()', ids), ids[0]);
  assert.equal(testIdentifier('Class/testExample', ids), undefined);
});
await test('unattributed issues, mismatched multisets, malformed wrappers and extra actions block', () => {
  for (const change of [
    (f: ReturnType<typeof setup>) => {
      record(f.invocation.actions)._values = [f.action, f.action];
    },
    (f: ReturnType<typeof setup>) => {
      record(record(f.invocation.metrics).testsCount)._value = '11.0';
    },
    (f: ReturnType<typeof setup>) => {
      delete record(f.invocation.issues).errorSummaries;
    },
    (f: ReturnType<typeof setup>) => {
      f.summary.testFailures = [];
    },
    (f: ReturnType<typeof setup>) => {
      f.outcomes.forEach((item) => {
        item.status = 'passed';
      });
    },
    (f: ReturnType<typeof setup>) => {
      const rows = record(record(f.invocation.issues).testFailureSummaries)._values;
      if (!Array.isArray(rows)) throw new Error('Invalid fixture');
      record(rows[0]).testCaseName = { _type: { _name: 'String' }, _value: 'Runner.finalizer()' };
    },
  ]) {
    const f = setup();
    change(f);
    assert.throws(() => classify(f));
  }
});
await test('nonterminal and unknown tree cases are not silently dropped', () => {
  const id = 'xctest:Target:Class/testCase';
  for (const result of ['Running', 'Expected Failure', 'Unknown'])
    assert.throws(() =>
      testOutcomes(
        { testNodes: [{ nodeType: 'Test Case', nodeIdentifier: 'Class/testCase', result }] },
        [id],
      ),
    );
  assert.throws(() =>
    testOutcomes(
      {
        testNodes: [{ nodeType: 'Test Case', nodeIdentifier: 'Other/testCase', result: 'Failed' }],
      },
      [id],
    ),
  );
});
