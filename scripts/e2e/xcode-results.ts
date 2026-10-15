interface TestOutcome {
  testId: string;
  status: 'passed' | 'failed' | 'skipped';
  durationMs: number;
}
function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value))
    throw new Error('Expected Xcode JSON object');
  return value as Record<string, unknown>;
}
function array(value: unknown): unknown[] {
  if (value === undefined) return [];
  if (!Array.isArray(value)) throw new Error('Expected Xcode JSON array');
  return value;
}
/** Reject enumeration errors even when xcodebuild exits zero. */
export function enumeratedTests(value: unknown): string[] {
  const data = object(value);
  const errors = array(data.errors);
  if (errors.length) throw new Error(`XCTest enumeration blocked: ${errors.join('; ')}`);
  const ids: string[] = [];
  for (const plan of array(data.values)) {
    for (const test of array(object(plan).enabledTests)) {
      const identifier = String(object(test).identifier);
      const parts = identifier.split('/');
      if (parts.length === 3 && parts[2].startsWith('test'))
        ids.push(`xctest:${parts[0]}:${parts[1]}/${parts[2].replace(/\(\)$/, '')}`);
    }
  }
  if (!ids.length || new Set(ids).size !== ids.length)
    throw new Error('No unique executable XCTest cases discovered');
  return ids;
}
export function testOutcomes(value: unknown, discovered: string[]): TestOutcome[] {
  const outcomes: TestOutcome[] = [];
  const visit = (value: unknown) => {
    const node = object(value);
    if (node.nodeType === 'Test Case') {
      const identifier = (
        typeof node.nodeIdentifier === 'string' ? node.nodeIdentifier : ''
      ).replace(/\(\)$/, '');
      const id = discovered.find((test) => test.endsWith(`:${identifier}`));
      if (id)
        outcomes.push({
          testId: id,
          status:
            node.result === 'Passed' ? 'passed' : node.result === 'Skipped' ? 'skipped' : 'failed',
          durationMs: Number(node.durationInSeconds ?? 0) * 1000,
        });
    }
    for (const child of array(node.children)) visit(child);
  };
  for (const node of array(object(value).testNodes)) visit(node);
  if (new Set(outcomes.map((item) => item.testId)).size !== outcomes.length)
    throw new Error('Duplicate XCTest outcomes; retries must be separate attempts');
  return outcomes;
}
/** A failed runner/finalizer can never certify a case that happened to report Passed. */
export function executionOutcome(
  outcome: Pick<TestOutcome, 'status'> | undefined,
  executionError: unknown,
) {
  const reasons: string[] = [];
  if (!outcome) reasons.push('No executable test outcome; inspect xcresult');
  if (executionError)
    reasons.push(
      'Native execution or evidence validation failed; inspect execution.log and provenance-validation.json',
    );
  return {
    status:
      outcome?.status === 'passed' && executionError ? 'blocked' : (outcome?.status ?? 'blocked'),
    environment: { valid: reasons.length === 0, reasons },
  };
}
