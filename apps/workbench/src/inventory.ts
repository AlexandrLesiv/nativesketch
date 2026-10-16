import type { ProjectSnapshot } from '../../../scripts/workbench/contracts/index.ts';
import { selectedHeading, el, notice, table } from './dom.ts';
import { filterForm, filtered, link, matches, selectionLink, route } from './route.ts';
import { sectionState, sourcePaths, identity } from './presentation.ts';
export function dependenciesView(snapshot: ProjectSnapshot): HTMLElement {
  const root = el('section');
  const data = snapshot.dependencies.data;
  root.append(
    sectionState(snapshot.dependencies),
    filterForm('dependencies', { ecosystem: data.map((row) => row.ecosystem) }),
    notice(
      'Inventory is not an advisory or license audit. Unknown metadata is not evidence of safety.',
    ),
    link('Read architecture boundary policy', 'docs', { selected: 'docs/architecture.md' }),
  );
  root.append(
    table(
      'Dependency inventory',
      [
        'Package',
        'Ecosystem',
        'Version / authority',
        'Scope',
        'License',
        'Provenance',
        'Resolved dependency IDs',
      ],
      data
        .filter((row) => matches(`${row.id} ${row.name}`) && filtered('ecosystem', row.ecosystem))
        .map((row) => [
          selectionLink(row.name, row.id),
          row.ecosystem,
          `${row.version ?? 'Unknown'} (${row.versionKind})`,
          row.scope,
          row.license ?? 'Unknown',
          row.provenance,
          row.dependencies.join(', ') || 'None recorded',
        ]),
    ),
  );
  const selected = data.find((row) => row.id === route().get('selected'));
  if (selected)
    root.append(
      selectedHeading(selected.name),
      el(
        'p',
        `Resolved relationships: ${selected.dependencies.join(', ') || 'None recorded'}. These do not define allowed architectural boundaries.`,
      ),
    );
  return root;
}
export function metricsView(snapshot: ProjectSnapshot): HTMLElement {
  const root = el('section');
  root.append(
    sectionState(snapshot.metrics),
    filterForm('metrics'),
    notice(
      'Measurements are not directly comparable without matching scope, configuration and source. Missing or invalid measurements are not zero.',
    ),
  );
  root.append(
    table(
      'Qualified metrics',
      ['Metric', 'Value', 'Status', 'Scope', 'Configuration', 'Captured'],
      snapshot.metrics.data
        .filter((row) => matches(`${row.label} ${row.scope}`))
        .map((row) => [
          selectionLink(row.label, row.id),
          row.status === 'measured' && row.value !== null
            ? `${row.value} ${row.unit}`
            : 'Unavailable',
          row.status,
          row.scope,
          row.configuration,
          row.capturedAt ?? 'Unknown',
        ]),
    ),
  );
  const selected = snapshot.metrics.data.find((row) => row.id === route().get('selected'));
  if (selected) {
    root.append(
      selectedHeading(selected.label),
      selected.source ? identity(selected.source) : notice('Source identity unknown'),
      sourcePaths(selected.evidencePaths),
    );
  }
  return root;
}
