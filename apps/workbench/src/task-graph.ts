import type { Task } from '../../../scripts/workbench/contracts/index.ts';
import { button, el, notice } from './dom.ts';
import { select } from './route.ts';
const ns = 'http://www.w3.org/2000/svg';
export function taskGraph(tasks: Task[]): HTMLElement {
  const root = el('section');
  root.append(
    el('h2', 'Dependency graph'),
    el('p', 'Arrows point prerequisite → dependent. The table below lists the same relationships.'),
  );
  if (tasks.length > 150) {
    root.append(notice('Graph limited to 150 tasks; use the full table.'));
    return root;
  }
  const ids = new Set(tasks.map((task) => task.id));
  const pending = new Map(tasks.map((task) => [task.id, task]));
  const ordered: Task[] = [];
  while (pending.size) {
    const next = [...pending.values()].find((task) =>
      task.depends_on.every((id) => !pending.has(id)),
    );
    if (!next)
      return (
        root.appendChild(
          notice(
            `Dependency cycle: ${[...pending.keys()].join(', ')}. Graph ordering unavailable; readable table retained.`,
          ),
        ).parentElement ?? root
      );
    pending.delete(next.id);
    ordered.push(next);
  }
  if (tasks.some((task) => task.depends_on.some((id) => !ids.has(id)))) {
    root.append(notice('Missing dependency IDs: graph unavailable; inspect the table.'));
    return root;
  }
  const region = el('div', '', 'graph-region');
  const svg = document.createElementNS(ns, 'svg');
  svg.setAttribute('role', 'group');
  svg.setAttribute('aria-label', 'Task dependencies');
  const width = 720,
    height = Math.max(80, ordered.length * 65);
  svg.setAttribute('viewBox', `0 0 ${width} ${height}`);
  svg.style.width = `${width}px`;
  svg.style.height = `${height}px`;
  const title = document.createElementNS(ns, 'title');
  title.textContent = 'Prerequisite to dependent; table equivalent follows';
  svg.append(title);
  for (const [index, task] of ordered.entries()) {
    for (const dependency of task.depends_on) {
      const start = ordered.findIndex((item) => item.id === dependency);
      const path = document.createElementNS(ns, 'path');
      path.setAttribute(
        'd',
        `M 580 ${start * 65 + 25} L 640 ${start * 65 + 25} L 640 ${index * 65 + 25} L 590 ${index * 65 + 25} l 7 -5 m -7 5 l 7 5`,
      );
      path.setAttribute('fill', 'none');
      path.setAttribute('stroke', 'currentColor');
      svg.append(path);
    }
    const anchor = document.createElementNS(ns, 'a');
    anchor.setAttribute('href', select(task.id));
    anchor.setAttribute('aria-label', `${task.id} ${task.title}: ${task.status}`);
    const rect = document.createElementNS(ns, 'rect');
    rect.setAttribute('x', '10');
    rect.setAttribute('y', String(index * 65 + 4));
    rect.setAttribute('width', '570');
    rect.setAttribute('height', '44');
    rect.setAttribute('rx', '6');
    rect.setAttribute('fill', 'var(--surface)');
    rect.setAttribute('stroke', 'currentColor');
    const text = document.createElementNS(ns, 'text');
    text.setAttribute('x', '20');
    text.setAttribute('y', String(index * 65 + 32));
    text.setAttribute('fill', 'currentColor');
    text.textContent = `${task.id} · ${task.title.slice(0, 48)} · ${task.status}`;
    anchor.append(rect, text);
    svg.append(anchor);
  }
  let zoom = 1;
  const resize = (factor: number) => {
    zoom = factor;
    svg.style.width = `${width * zoom}px`;
    svg.style.height = `${height * zoom}px`;
  };
  root.append(
    button('Zoom graph in', () => resize(Math.min(2, zoom + 0.25))),
    button('Zoom graph out', () => resize(Math.max(0.5, zoom - 0.25))),
    button('Reset graph zoom', () => resize(1)),
  );
  region.append(svg);
  root.append(region);
  return root;
}
