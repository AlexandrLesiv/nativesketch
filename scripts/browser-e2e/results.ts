import type { JSONReport, JSONReportSpec, JSONReportSuite } from 'playwright/types/testReporter';
export function browserSpecs(
  report: Pick<JSONReport, 'suites' | 'errors'>,
  identify: (spec: JSONReportSpec) => string = browserTestId,
) {
  if (!Array.isArray(report.suites) || !Array.isArray(report.errors) || report.errors.length)
    throw new Error('Playwright discovery/report has global errors');
  const specs: JSONReportSpec[] = [];
  const walk = (suites: JSONReportSuite[]) => {
    for (const suite of suites) {
      if (!Array.isArray(suite.specs)) throw new Error('Invalid Playwright suite');
      specs.push(...suite.specs);
      if (suite.suites) walk(suite.suites);
    }
  };
  walk(report.suites);
  const ids = specs.map(identify);
  if (!ids.length || new Set(ids).size !== ids.length)
    throw new Error('Empty or duplicate browser discovery');
  for (const spec of specs) {
    if (
      spec.tests.length !== 1 ||
      spec.tests[0].projectName !== 'chromium' ||
      spec.tests[0].expectedStatus !== 'passed'
    )
      throw new Error('Unexpected browser project or expected-failure test');
  }
  return specs;
}
export function browserTestId(spec: JSONReportSpec) {
  const id = /^WEB-(?:[A-Z]+-)+\d{3}(?=\s|$)/.exec(spec.title)?.[0];
  if (!id) throw new Error('Browser test needs a stable acceptance ID');
  return `playwright:chromium:${id}`;
}
export function browserOutcome(spec: JSONReportSpec) {
  const runs = spec.tests[0].results;
  if (runs.length !== 1 || runs[0].retry !== 0)
    throw new Error('Missing or unreported browser retry');
  const run = runs[0];
  if (
    !run.status ||
    !['passed', 'failed', 'timedOut', 'skipped', 'interrupted'].includes(run.status)
  )
    throw new Error('Missing or unknown run status');
  const reason = [run.error?.message ?? '', ...run.errors.map((error) => error.message ?? '')].join(
    '\n',
  );
  const invalid =
    /browserType\.launch|Executable doesn't exist|Target page, context or browser has been closed|worker process exited unexpectedly/i.test(
      reason,
    );
  const status =
    invalid || run.status === 'interrupted'
      ? 'blocked'
      : run.status === 'passed'
        ? 'passed'
        : run.status === 'skipped'
          ? 'skipped'
          : 'failed';
  return {
    run,
    status,
    environment: {
      valid: !invalid && run.status !== 'interrupted',
      reasons: invalid
        ? [reason]
        : run.status === 'interrupted'
          ? ['Browser execution interrupted']
          : [],
    },
    output: reason,
  };
}
