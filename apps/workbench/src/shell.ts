import type { ProjectSnapshot } from '../../../scripts/workbench/contracts/index.ts';
import { button, detail, el, notice } from './dom.ts';
import { currentView, destinations, link, views } from './route.ts';
import { identity } from './presentation.ts';
export function shell(
  snapshot: ProjectSnapshot,
  live: boolean,
  connected: boolean,
  refresh: () => void,
): { root: HTMLElement; main: HTMLElement; status: HTMLElement } {
  const root = el('div');
  const skip = el('a', 'Skip to content', 'skip');
  skip.href = '#main-content';
  skip.addEventListener('click', (event) => {
    event.preventDefault();
    document.getElementById('main-content')?.focus();
  });
  const header = el('header');
  header.append(
    el('p', 'NATIVE SKETCH', 'eyebrow'),
    el('h1', 'Project workbench'),
    el(
      'p',
      `${snapshot.branch} · ${snapshot.source.revision.slice(0, 12)} · ${snapshot.source.dirtyDiffSha256 ? 'Dirty' : 'Clean'}`,
    ),
    el(
      'p',
      `${live ? 'Local workbench' : 'Static report · read-only'} · Captured ${snapshot.generatedAt}`,
    ),
    el('p', `Host: ${snapshot.host.platform} / ${snapshot.host.architecture}`),
    identity(snapshot.source),
  );
  const refreshButton = button('Refresh snapshot', refresh);
  refreshButton.disabled = !live || !connected;
  header.append(refreshButton);
  const status = el('p');
  status.role = 'status';
  status.setAttribute('aria-live', 'polite');
  const nav = el('nav');
  nav.setAttribute('aria-label', 'Workbench destinations');
  for (const [index, view] of views.entries()) {
    const anchor = link(destinations[index] ?? view, view);
    if (view === currentView()) anchor.setAttribute('aria-current', 'page');
    nav.append(anchor);
  }
  const disclosure = detail('Navigate workbench', nav);
  disclosure.open = true;
  disclosure.className = 'navigation';
  const main = el('main');
  main.id = 'main-content';
  main.tabIndex = -1;
  main.append(el('h2', destinations[views.indexOf(currentView())] ?? 'Overview'));
  const layout = el('div', '', 'layout');
  layout.append(disclosure, main);
  root.append(
    skip,
    header,
    notice(
      'Foreground actions are disabled. No app launches, native tests, GPU benchmarks or IDE opening.',
    ),
    status,
    layout,
  );
  return { root, main, status };
}
