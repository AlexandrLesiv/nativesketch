import type { ProjectSnapshot } from '../../../scripts/workbench/contracts/index.ts';
import { el, notice } from './dom.ts';
import { link } from './route.ts';
import { sectionState } from './presentation.ts';
export function overviewView(snapshot: ProjectSnapshot): HTMLElement {
  const root = el('section');
  root.append(
    el('h2', 'Project evidence at a glance'),
    notice(
      'Implementation, execution evidence and owner acceptance are separate. This report does not infer release readiness.',
    ),
  );
  const cards = el('div', '', 'cards');
  for (const [title, count, target, state] of [
    ['Board tasks', snapshot.tasks.data.length, 'tasks', snapshot.tasks.state],
    ['Indexed documents', snapshot.documents.data.length, 'docs', snapshot.documents.state],
    [
      'Registered requirements loaded',
      snapshot.coverage.data.rows.length,
      'coverage',
      snapshot.coverage.state,
    ],
    ['Qualified metric records', snapshot.metrics.data.length, 'metrics', snapshot.metrics.state],
  ] as const) {
    const card = el('section', '', 'card');
    card.append(
      el('h3', title),
      el(
        'p',
        state === 'invalid'
          ? 'Unavailable — validation failed'
          : state === 'missing' && count === 0
            ? 'Unavailable — data missing'
            : String(count),
        state === 'invalid' || (state === 'missing' && count === 0) ? '' : 'count',
      ),
      link(`View ${title.toLowerCase()}`, target),
    );
    cards.append(card);
  }
  root.append(cards, el('h2', 'Snapshot limitations'));
  for (const [name, section] of Object.entries({
    tasks: snapshot.tasks,
    documents: snapshot.documents,
    dependencies: snapshot.dependencies,
    coverage: snapshot.coverage,
    metrics: snapshot.metrics,
    actions: snapshot.actions,
  })) {
    const item = el('section');
    item.append(el('h3', `${name}: ${section.state}`), sectionState(section));
    root.append(item);
  }
  root.append(
    notice(`Evidence freshness: ${snapshot.coverage.data.freshness}`),
    link('Inspect coverage gaps and attempts', 'coverage'),
  );
  return root;
}
