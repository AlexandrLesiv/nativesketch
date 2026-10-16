export type Content = Node | string;
export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  text = '',
  className = '',
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  node.textContent = text;
  node.className = className;
  return node;
}
export function button(label: string, action: () => void): HTMLButtonElement {
  const node = el('button', label);
  node.type = 'button';
  node.addEventListener('click', action);
  return node;
}
export function detail(label: string, ...children: Content[]): HTMLDetailsElement {
  const node = el('details');
  node.append(el('summary', label), ...children);
  return node;
}
export function table(caption: string, headings: string[], rows: Content[][]): HTMLElement {
  const region = el('div', '', 'table-region');
  region.role = 'region';
  region.setAttribute('aria-label', caption);
  region.tabIndex = 0;
  region.append(el('p', 'Scroll horizontally to view every column when needed.', 'muted'));
  const node = el('table');
  node.append(el('caption', caption));
  const head = el('tr');
  for (const name of headings) {
    const cell = el('th', name);
    cell.scope = 'col';
    head.append(cell);
  }
  const thead = el('thead');
  thead.append(head);
  const body = el('tbody');
  for (const row of rows) {
    const tr = el('tr');
    for (const value of row) {
      const td = el('td');
      td.append(value);
      tr.append(td);
    }
    body.append(tr);
  }
  node.append(thead, body);
  region.append(node);
  return region;
}
export function textList(items: string[]): HTMLElement {
  const list = el('ul');
  for (const item of items) list.append(el('li', item));
  return list;
}
export function notice(message: string): HTMLElement {
  return el('p', message, 'notice');
}

export function selectedHeading(text: string): HTMLHeadingElement {
  const heading = el('h2', text);
  heading.id = 'selected-detail';
  heading.tabIndex = -1;
  return heading;
}
