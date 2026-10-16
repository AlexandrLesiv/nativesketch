import type { Source, Feature, Discovery, Execution, Result } from './types.ts';
import {
  validateDiscovery,
  validateExecution,
  validateFeatures,
  requireValue,
} from './validate.ts';

import { matchesRequirement, mappingIssues, wasEnumerated } from './mappings.ts';

const sameSource = (a: Source, b: Source) =>
  a.revision === b.revision && a.dirtyDiffSha256 === b.dirtyDiffSha256;

export function buildReport(features: Feature[], discovery: Discovery, executions: Execution[]) {
  validateFeatures(features);
  validateDiscovery(discovery);
  const tests = new Map(discovery.tests.map((test) => [test.id, test]));
  const issues = mappingIssues(features, discovery);

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
        const coverage = requirement.coverage ?? item;
        const absent = requirement.coverage?.testIds.filter((id) => !tests.has(id)) ?? [];
        const incompatible = requirement.coverage?.testIds.some((id) => {
          const test = tests.get(id);
          return test && !matchesRequirement(test, requirement);
        });
        const eligible = coverage.testIds.filter((id) => {
          const test = tests.get(id);
          return test && matchesRequirement(test, requirement);
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
          (incompatible ? 'invalid-mapping' : undefined) ??
          (absent.length
            ? absent.some((id) => wasEnumerated(id, requirement, discovery))
              ? 'invalid-mapping'
              : 'missing-discovery'
            : undefined) ??
          statuses.find((s) => s !== 'passed') ??
          (!eligible.length
            ? 'missing'
            : coverage.gaps.length || coverage.automation !== 'implemented'
              ? 'gap'
              : 'passed');
        return {
          feature: feature.id,
          caseId: item.id,
          expected: item.expected,
          platform: requirement.platform,
          level: requirement.level,
          pointerWidth: requirement.pointerWidth,
          automation: coverage.automation,
          status,
          testIds: requirement.coverage ? coverage.testIds : eligible,
          discoveredTestIds: eligible,
          gaps: coverage.gaps,
          attempts: eligible.flatMap((id) => (attempts.get(id) ?? []).map((r) => r.result)),
        };
      }),
    ),
  );
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
