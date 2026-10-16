import { ProcessFailure } from '../lib/process-failure.ts';
import { record, wrapped, list, string, integer, count } from './xcresult-values.ts';
import { testIdentifier } from './test-identifiers.ts';
import type { TestOutcome } from './xcode-results.ts';

function equal(actual: unknown, expected: unknown) {
  if (JSON.stringify(actual) !== JSON.stringify(expected))
    throw new Error('Inconsistent XCTest execution evidence');
}
function metrics(value: unknown, expected: number[]) {
  const data = wrapped(value, 'ResultMetrics');
  equal(integer(data.errorCount), 0);
  equal(
    ['testsCount', 'testsFailedCount', 'testsSkippedCount'].map((key) => integer(data[key])),
    expected,
  );
}
function failures(value: unknown, selected: string[], failed: string[]) {
  const issues = wrapped(value, 'ResultIssueSummaries');
  equal(list(issues.errorSummaries), []);
  const rows = list(issues.testFailureSummaries).map((value) => {
    const issue = wrapped(value, 'TestFailureIssueSummary');
    const id = testIdentifier(string(issue.testCaseName), selected);
    if (!id || !failed.includes(id)) throw new Error('Unattributed XCTest failure');
    const location =
      issue.documentLocationInCreatingWorkspace === undefined
        ? ''
        : string(record(issue.documentLocationInCreatingWorkspace).url);
    return { id, message: string(issue.message), type: string(issue.issueType), location };
  });
  equal([...new Set(rows.map((row) => row.id))].sort(), [...failed].sort());
  return rows.map((row) => JSON.stringify(row)).sort();
}

/** Exit 65 alone proves nothing. Accept only fully reconciled, completed test failures. */
export function completedTestFailures(
  error: unknown,
  invocation: unknown,
  summary: unknown,
  outcomes: TestOutcome[],
  selected: string[],
) {
  if (!(error instanceof ProcessFailure) || error.exitCode !== 65 || error.signal || error.reason)
    throw new Error('Not a normal XCTest failure exit');
  if (
    new Set(selected).size !== selected.length ||
    new Set(outcomes.map((item) => item.testId)).size !== outcomes.length
  )
    throw new Error('Duplicate selected tests or outcomes');
  equal(outcomes.map((item) => item.testId).sort(), [...selected].sort());
  const failed = outcomes.filter((item) => item.status === 'failed').map((item) => item.testId);
  const skipped = outcomes.filter((item) => item.status === 'skipped').length;
  if (!failed.length) throw new Error('Exit 65 without failed tests');
  const expected = [selected.length, failed.length, skipped];
  const root = wrapped(invocation, 'ActionsInvocationRecord');
  const actions = list(root.actions);
  if (actions.length !== 1) throw new Error('Expected exactly one XCTest action');
  const action = wrapped(actions[0], 'ActionRecord');
  equal(string(action.schemeCommandName), 'Test');
  const result = wrapped(action.actionResult, 'ActionResult');
  equal(string(result.status), 'failed');
  const build = wrapped(action.buildResult, 'ActionResult');
  equal(string(build.status), 'notRequested');
  metrics(build.metrics, [0, 0, 0]);
  equal(failures(build.issues, selected, []), []);
  const start = Date.parse(string(action.startedTime, 'Date'));
  const finish = Date.parse(string(action.endedTime, 'Date'));
  if (!Number.isFinite(start) || !Number.isFinite(finish) || finish < start)
    throw new Error('Incomplete XCTest action timestamps');
  metrics(root.metrics, expected);
  metrics(result.metrics, expected);
  equal(failures(root.issues, selected, failed), failures(result.issues, selected, failed));
  const modern = record(summary);
  equal(modern.result, 'Failed');
  equal(count(modern.expectedFailures), 0);
  equal(
    [count(modern.totalTestCount), count(modern.failedTests), count(modern.skippedTests)],
    expected,
  );
  equal(count(modern.passedTests), selected.length - failed.length - skipped);
  if (
    typeof modern.startTime !== 'number' ||
    typeof modern.finishTime !== 'number' ||
    !Number.isFinite(modern.startTime) ||
    !Number.isFinite(modern.finishTime) ||
    Math.abs(modern.startTime * 1000 - start) > 1 ||
    Math.abs(modern.finishTime * 1000 - finish) > 1
  )
    throw new Error('Inconsistent XCTest summary timestamps');
  if (!Array.isArray(modern.testFailures)) throw new Error('Missing summary failures');
  const ids = modern.testFailures.map((value: unknown) => {
    const row = record(value);
    if (
      typeof row.targetName !== 'string' ||
      typeof row.testIdentifierString !== 'string' ||
      typeof row.failureText !== 'string' ||
      !row.failureText
    )
      throw new Error('Missing summary failure identity');
    const id = testIdentifier(`${row.targetName}/${row.testIdentifierString}`, selected);
    if (!id || !failed.includes(id)) throw new Error('Unattributed summary failure');
    return id;
  });
  equal([...new Set(ids)].sort(), [...failed].sort());
  return { kind: 'completed-test-failures', exitCode: 65 };
}
