import type { JSONReportSpec } from 'playwright/types/testReporter';
import { ProcessFailure } from '../lib/process-failure.ts';
import { browserOutcome } from './results.ts';

export type ProcessResult = {
  exitCode: number | null;
  signal: NodeJS.Signals | null;
  reason: string;
};

export function failedProcess(error: unknown): ProcessResult {
  return error instanceof ProcessFailure
    ? { exitCode: error.exitCode, signal: error.signal, reason: error.reason }
    : { exitCode: null, signal: null, reason: String(error) };
}

export function processSucceeded(result: ProcessResult) {
  return result.exitCode === 0 && result.signal === null && result.reason === '';
}

/** A JSON report alone cannot certify that the runner finished normally. */
export function executionReasons(result: ProcessResult, specs: JSONReportSpec[]): string[] {
  if (result.reason || result.signal || ![0, 1].includes(result.exitCode ?? -1))
    return [result.reason || `Browser process exit ${result.exitCode}, signal ${result.signal}`];
  const statuses = specs.map((spec) => browserOutcome(spec).run.status);
  const failures = statuses.some((status) => status === 'failed' || status === 'timedOut');
  if (result.exitCode === 1 && !failures)
    return ['Browser process failed without an attributable test failure'];
  if (result.exitCode === 0 && (failures || statuses.includes('interrupted')))
    return ['Browser process success disagrees with test outcomes'];
  return [];
}

export function certifiedOutcome(spec: JSONReportSpec, reasons: string[]) {
  const outcome = browserOutcome(spec);
  return reasons.length
    ? { ...outcome, status: 'blocked', environment: { valid: false, reasons } }
    : outcome;
}
