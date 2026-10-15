import type { Source, Feature, Discovery, Execution, Result } from './types.ts';
import {
  validateDiscovery,
  validateExecution,
  validateFeatures,
  requireValue,
} from './validate.ts';

const sameSource = (a: Source, b: Source) =>
  a.revision === b.revision && a.dirtyDiffSha256 === b.dirtyDiffSha256;

export function buildReport(features: Feature[], discovery: Discovery, executions: Execution[]) {
  validateFeatures(features);
  validateDiscovery(discovery);
  const tests = new Map(discovery.tests.map((test) => [test.id, test]));
  const mapped = new Set(features.flatMap((f) => f.cases.flatMap((c) => c.testIds)));
  const issues = [];
  for (const id of mapped) if (!tests.has(id)) issues.push(`Undiscovered required test: ${id}`);

  const attempts: Map<
    string,
    {
      result: Result;
      stale: boolean;
    }[]
  > = new Map();
  for (const execution of executions) {
    validateExecution(execution);
    for (const result of execution.results) {
      const test = tests.get(result.testId);
      requireValue(test, `Unknown result ID: ${result.testId}`);
      const rows = attempts.get(result.testId) ?? [];
      requireValue(
        !rows.some((row) => row.result.attempt === result.attempt),
        `Duplicate attempt: ${result.testId}`,
      );
      rows.push({ result, stale: !sameSource(discovery.source, execution.source) });
      attempts.set(result.testId, rows);
    }
  }
  const rows = features.flatMap((feature) =>
    feature.cases.flatMap((item) =>
      item.requirements.map((requirement) => {
        const eligible = item.testIds.filter((id) => {
          const test = tests.get(id);
          return (
            test &&
            test.platform === requirement.platform &&
            test.level === requirement.level &&
            (requirement.pointerWidth === undefined ||
              test.pointerWidth === requirement.pointerWidth)
          );
        });
        const statuses = eligible.map((id) => {
          const test = tests.get(id);
          const runs = attempts.get(id) ?? [];
          if (!runs.length) return 'missing';
          if (runs.some((r) => r.stale)) return 'stale';
          if (
            runs.some(
              ({ result: r }) =>
                r.binarySha256 !== test?.binarySha256 ||
                r.testBinarySha256 !== test?.testBinarySha256 ||
                r.platform !== test?.platform,
            )
          )
            return 'mismatched';
          if (runs.some(({ result: r }) => !r.environment.valid)) return 'invalid-environment';
          const states = runs.map((r) => r.result.status);
          if (states.includes('failed') && states.includes('passed')) return 'flaky';
          return states.find((state) => state !== 'passed') ?? 'passed';
        });
        const status =
          statuses.find((s) => s !== 'passed') ??
          (!eligible.length
            ? 'missing'
            : item.gaps.length || item.automation !== 'implemented'
              ? 'gap'
              : 'passed');
        return {
          feature: feature.id,
          caseId: item.id,
          expected: item.expected,
          ...requirement,
          automation: item.automation,
          status,
          testIds: eligible,
          gaps: item.gaps,
          attempts: eligible.flatMap((id) => (attempts.get(id) ?? []).map((r) => r.result)),
        };
      }),
    ),
  );
  for (const item of features.flatMap((f) => f.cases)) {
    for (const id of item.testIds) {
      const test = tests.get(id);
      if (
        test &&
        !item.requirements.some((r) => r.platform === test.platform && r.level === test.level)
      )
        issues.push(`Orphan mapping: ${item.id} -> ${id}`);
    }
  }
  return {
    schemaVersion: 1,
    source: discovery.source,
    issues,
    rows,
    passed: issues.length === 0 && rows.every((row) => row.status === 'passed'),
  };
}

export const escapeHtml = (value: unknown) =>
  String(value).replace(
    /[&<>"']/g,
    (char) =>
      ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[char] ?? char,
  );

export function renderHtml(report: ReturnType<typeof buildReport>) {
  const rows = report.rows
    .map(
      (row) =>
        `<tr><td>${escapeHtml(row.caseId)}<br>${escapeHtml(row.expected)}</td><td>${escapeHtml(row.platform)} / ${escapeHtml(row.level)}</td><td>${escapeHtml(row.automation)}</td><td>${escapeHtml(row.status)}</td><td>${row.gaps.map((g) => escapeHtml(g.reason)).join('<br>')}<br>${row.attempts.map((r) => `Attempt ${r.attempt}: ${escapeHtml(r.status)} ${r.artifacts.map((p) => `<a href="${escapeHtml(p.split('/').map(encodeURIComponent).join('/'))}">${escapeHtml(p)}</a>`).join(' ')}`).join('<br>')}</td></tr>`,
    )
    .join('\n');
  return `<!doctype html><html lang="en"><meta charset="utf-8"><title>Feature evidence</title><style>body{font:16px system-ui;margin:2rem}table{border-collapse:collapse;width:100%}th,td{border:1px solid #aaa;padding:.6rem;text-align:left;vertical-align:top}th{background:#eee}</style><h1>Feature evidence</h1><p>Source ${escapeHtml(report.source.revision)}; dirty digest ${escapeHtml(report.source.dirtyDiffSha256 ?? 'clean')}. Overall: ${report.passed ? 'passed' : 'coverage incomplete'}. Automated implementation and execution are separate. All attempts retained.</p><p>${report.issues.map(escapeHtml).join('<br>')}</p><table><tr><th>Acceptance</th><th>Platform / level</th><th>Automation</th><th>Execution</th><th>Gaps and evidence</th></tr>${rows}</table></html>`;
}
