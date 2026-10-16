import type { ProjectSnapshot, CoverageRow } from '../../../scripts/workbench/contracts/index.ts';
import { selectedHeading, detail, el, notice, table, textList } from './dom.ts';
import { filterForm, filtered, matches, route, selectionLink } from './route.ts';
import { sectionState, identity, jsonDetail, sourcePaths } from './presentation.ts';
function key(row: CoverageRow): string {
  return `${row.caseId}/${row.platform}/${row.level}/${row.pointerWidth ?? ''}`;
}
export function coverageView(snapshot: ProjectSnapshot): HTMLElement {
  const root = el('section');
  const data = snapshot.coverage.data;
  root.append(
    sectionState(snapshot.coverage),
    notice(
      `Evidence source freshness: ${data.freshness}. Recorded pass is not a current-source pass when stale or unknown.`,
    ),
  );
  root.append(
    data.discoverySource
      ? identity(data.discoverySource)
      : notice('No discovery source loaded. Missing execution evidence is not zero failures.'),
    sourcePaths(data.evidencePaths),
    ...(data.issues.length
      ? [detail(`Mapping issues (${data.issues.length})`, textList(data.issues))]
      : []),
  );
  root.append(
    filterForm('coverage', {
      platform: data.rows.map((row) => row.platform),
      level: data.rows.map((row) => row.level),
      status: data.rows.map((row) => row.status),
      ownerTask: data.rows.map((row) => row.ownerTask),
    }),
  );
  const rows = data.rows.filter(
    (row) =>
      matches(`${row.caseId} ${row.feature} ${row.expected}`) &&
      filtered('platform', row.platform) &&
      filtered('level', row.level) &&
      filtered('status', row.status) &&
      filtered('ownerTask', row.ownerTask),
  );
  root.append(
    table(
      'Acceptance requirements, not line coverage',
      [
        'Case / feature',
        'Platform',
        'Required level',
        'Automation',
        'Recorded execution',
        'Source freshness',
        'Owner',
      ],
      rows.map((row) => [
        selectionLink(`${row.caseId} · ${row.feature}`, key(row)),
        row.platform,
        row.level,
        row.automation,
        row.status,
        data.freshness,
        row.ownerTask,
      ]),
    ),
  );
  const selected = data.rows.find((row) => key(row) === route().get('selected'));
  if (selected) {
    root.append(
      selectedHeading(selected.caseId),
      el('p', selected.expected),
      textList(selected.invariants),
      detail(
        'Mapped and discovered tests',
        textList(selected.testIds.map((id) => `Mapped: ${id}`)),
        textList(selected.discoveredTestIds.map((id) => `Discovered: ${id}`)),
      ),
      textList(selected.gaps.map((gap) => `${gap.reason} — ${gap.ownerTask}`)),
    );
    if (!selected.attempts.length) root.append(notice('No execution attempts recorded.'));
    for (const attempt of selected.attempts)
      root.append(
        jsonDetail(
          `${attempt.testId} · attempt ${attempt.attempt} · ${attempt.status} · environment ${attempt.environment.valid ? 'valid' : 'invalid'}`,
          attempt,
        ),
      );
  }
  if (!rows.length)
    root.append(
      notice(
        snapshot.coverage.state === 'invalid'
          ? 'Requirements unavailable because coverage validation failed.'
          : snapshot.coverage.state === 'missing' && !data.rows.length
            ? 'Requirements unavailable because coverage data is missing.'
            : 'No requirements match these filters.',
      ),
    );
  return root;
}
