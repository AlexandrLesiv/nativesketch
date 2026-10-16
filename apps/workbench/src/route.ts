import { el } from './dom.ts';
export const destinations = [
  'Overview',
  'Tasks',
  'Docs',
  'Dependencies',
  'Coverage',
  'Actions & jobs',
  'Metrics',
] as const;
export type View =
  | 'overview'
  | 'tasks'
  | 'docs'
  | 'dependencies'
  | 'coverage'
  | 'actions'
  | 'metrics';
export const views: View[] = [
  'overview',
  'tasks',
  'docs',
  'dependencies',
  'coverage',
  'actions',
  'metrics',
];
export function route(): URLSearchParams {
  return new URLSearchParams(location.hash.slice(1));
}
export function currentView(): View {
  const value = route().get('view');
  return views.find((view) => view === value) ?? 'overview';
}
export function href(view: View, fields: Record<string, string> = {}): string {
  return `#${new URLSearchParams({ view, ...fields })}`;
}
export function link(
  label: string,
  view: View,
  fields: Record<string, string> = {},
): HTMLAnchorElement {
  const node = el('a', label);
  node.href = href(view, fields);
  return node;
}
export function select(id: string): string {
  const params = route();
  params.set('selected', id);
  return `#${params}`;
}
export function selectionLink(label: string, id: string): HTMLAnchorElement {
  const node = el('a', label);
  node.href = select(id);
  return node;
}
export function matches(text: string): boolean {
  return text.toLocaleLowerCase().includes((route().get('q') ?? '').toLocaleLowerCase());
}
export function filterForm(label: string, filters: Record<string, string[]> = {}): HTMLFormElement {
  const form = el('form', '', 'filters');
  const searchLabel = el('label', `Search ${label}`);
  const search = el('input');
  search.type = 'search';
  search.name = 'q';
  search.value = route().get('q') ?? '';
  searchLabel.append(search);
  form.append(searchLabel);
  for (const [name, options] of Object.entries(filters)) {
    const labelNode = el('label', name);
    const field = el('select');
    field.name = name;
    for (const value of ['', ...new Set(options)]) {
      const option = el('option', value || 'All');
      option.value = value;
      field.append(option);
    }
    field.value = route().get(name) ?? '';
    labelNode.append(field);
    form.append(labelNode);
  }
  const apply = el('button', 'Apply filters');
  apply.type = 'submit';
  form.append(apply);
  form.addEventListener('submit', (event) => {
    event.preventDefault();
    const params = new URLSearchParams({ view: currentView() });
    for (const [name, value] of new FormData(form))
      if (typeof value === 'string' && value) params.set(name, value);
    location.hash = params.toString();
  });
  return form;
}
export function filtered(name: string, value: string): boolean {
  return !route().get(name) || route().get(name) === value;
}
