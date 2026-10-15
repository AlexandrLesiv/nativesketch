/** @param {unknown} value @returns {Record<string, any>} */
function object(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value))
    throw new Error('Expected Xcode JSON object');
  return /** @type {Record<string, any>} */ (value);
}
/** Reject enumeration errors even when xcodebuild exits zero. @param {unknown} value */
export function enumeratedTests(value) {
  const data = object(value);
  if (data.errors?.length) throw new Error(`XCTest enumeration blocked: ${data.errors.join('; ')}`);
  const ids = /** @type {string[]} */ ([]);
  for (const plan of data.values ?? []) {
    for (const test of plan.enabledTests ?? []) {
      const identifier = String(test.identifier);
      // Flat executable enumeration includes target/class/method, not just target names.
      const parts = identifier.split('/');
      if (parts.length === 3 && parts[2].startsWith('test'))
        ids.push(`xctest:${parts[0]}:${parts[1]}/${parts[2].replace(/\(\)$/, '')}`);
    }
  }
  if (!ids.length || new Set(ids).size !== ids.length)
    throw new Error('No unique executable XCTest cases discovered');
  return ids;
}
/** @param {unknown} value @param {string[]} discovered */
export function testOutcomes(value, discovered) {
  const data = object(value);
  const outcomes =
    /** @type {{testId:string,status:'passed'|'failed'|'skipped',durationMs:number}[]} */ ([]);
  const visit = (/** @type {Record<string, any>} */ node) => {
    if (node.nodeType === 'Test Case') {
      const identifier = String(node.nodeIdentifier ?? '').replace(/\(\)$/, '');
      const id = discovered.find((test) => test.endsWith(`:${identifier}`));
      if (id) {
        const status =
          node.result === 'Passed' ? 'passed' : node.result === 'Skipped' ? 'skipped' : 'failed';
        outcomes.push({
          testId: id,
          status,
          durationMs: Number(node.durationInSeconds ?? 0) * 1000,
        });
      }
    }
    for (const child of node.children ?? []) visit(child);
  };
  for (const node of data.testNodes ?? []) visit(node);
  if (new Set(outcomes.map((item) => item.testId)).size !== outcomes.length)
    throw new Error('Duplicate XCTest outcomes; retries must be separate attempts');
  return outcomes;
}

/** A failed runner/finalizer can never certify a case that happened to report Passed.
 * @param {{status:'passed'|'failed'|'skipped'}|undefined} outcome @param {unknown} executionError */
export function executionOutcome(outcome, executionError) {
  const reasons = [];
  if (!outcome) reasons.push('No executable test outcome; inspect xcresult');
  if (executionError) reasons.push('xcodebuild execution failed; inspect execution.log');
  return {
    status:
      outcome?.status === 'passed' && executionError ? 'blocked' : (outcome?.status ?? 'blocked'),
    environment: { valid: reasons.length === 0, reasons },
  };
}
