import type { ProjectSnapshot } from '../../../scripts/workbench/contracts/index.ts';
import { selectedHeading, el, notice } from './dom.ts';
import { markdown } from './markdown.ts';
import { filterForm, matches, route, selectionLink } from './route.ts';
import { sectionState } from './presentation.ts';
export function docsView(snapshot: ProjectSnapshot): HTMLElement {
  const root = el('section');
  root.append(sectionState(snapshot.documents), filterForm('documents'));
  const list = el('ul');
  for (const doc of snapshot.documents.data.filter((doc) =>
    matches(`${doc.id} ${doc.title} ${doc.text}`),
  )) {
    const item = el('li');
    item.append(selectionLink(`${doc.title} — ${doc.id}`, doc.id));
    if (route().get('q')) {
      const offset = doc.text
        .toLocaleLowerCase()
        .indexOf((route().get('q') ?? '').toLocaleLowerCase());
      item.append(
        el('p', doc.text.slice(Math.max(0, offset - 40), Math.max(0, offset) + 150), 'muted'),
      );
    }
    list.append(item);
  }
  root.append(list);
  const selected = snapshot.documents.data.find((doc) => doc.id === route().get('selected'));
  if (selected) {
    root.append(
      selectedHeading(selected.title),
      el('p', selected.id),
      el('p', `Content SHA-256: ${selected.sha256}`),
      markdown(selected, snapshot.documents.data),
    );
  } else
    root.append(
      notice(
        route().has('selected')
          ? 'Not in this snapshot'
          : 'Select a document to read it. Raw HTML and embedded scripts are displayed as text.',
      ),
    );
  return root;
}
