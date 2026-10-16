import type { ProjectSnapshot } from '../../../scripts/workbench/contracts/index.ts';
import { selectedHeading, detail, el, notice, table, textList } from './dom.ts';
import { filterForm, filtered, link, matches, route, selectionLink } from './route.ts';
import { sectionState } from './presentation.ts';
import { taskGraph } from './task-graph.ts';
export function tasksView(snapshot: ProjectSnapshot): HTMLElement {
  const root = el('section');
  const tasks = snapshot.tasks.data;
  root.append(
    sectionState(snapshot.tasks),
    filterForm('tasks', {
      status: tasks.map((task) => task.status),
      owner: tasks.flatMap((task) => task.roles),
    }),
  );
  root.append(
    notice('Board status is owner-maintained. Technical evidence does not imply owner acceptance.'),
  );
  root.append(detail('Show dependency graph', taskGraph(tasks)));
  const visible = tasks.filter(
    (task) =>
      matches(`${task.id} ${task.title}`) &&
      filtered('status', task.status) &&
      (!route().get('owner') || task.roles.includes(route().get('owner') ?? '')),
  );
  root.append(
    table(
      'Tasks and equivalent dependency list',
      [
        'Task',
        'Board status',
        'Owner roles',
        'Prerequisites → this task',
        'Dependents',
        'Technical evidence',
      ],
      visible.map((task) => {
        const prerequisites = el('div');
        for (const id of task.depends_on) prerequisites.append(selectionLink(id, id), ' ');
        const dependents = el('div');
        for (const other of tasks.filter((item) => item.depends_on.includes(task.id)))
          dependents.append(selectionLink(other.id, other.id), ' ');
        const rows = snapshot.coverage.data.rows.filter((row) => row.ownerTask === task.id);
        return [
          selectionLink(`${task.id} ${task.title}`, task.id),
          task.status,
          task.roles.join(', '),
          prerequisites,
          dependents,
          rows.length
            ? `${rows.length} mapped requirements; ${snapshot.coverage.data.freshness} source — see Coverage`
            : 'No mapped evidence loaded',
        ];
      }),
    ),
  );
  const task = tasks.find((item) => item.id === route().get('selected'));
  if (task) {
    root.append(
      selectedHeading(`${task.id} ${task.title}`),
      el('p', `Board status: ${task.status}`),
      link('Read task brief', 'docs', { selected: task.brief }),
      el('p', `Source: ${task.brief}`),
      textList(task.depends_on.map((id) => `Prerequisite: ${id}`)),
      link('View mapped coverage', 'coverage', { ownerTask: task.id }),
    );
  }
  if (!visible.length) root.append(notice('No tasks match these filters.'));
  return root;
}
