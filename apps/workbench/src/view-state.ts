/** Preserve inspection state across live job refreshes without keeping service state in the DOM. */
function key(node: Element): string {
  if (
    node instanceof HTMLElement &&
    (node.dataset.log || node.dataset.inspection || node.dataset.job)
  )
    return (
      node.tagName +
      ':' +
      (node.closest<HTMLElement>('[data-job]')?.dataset.job ?? '') +
      ':' +
      (node.dataset.log ?? node.dataset.inspection ?? 'job')
    );
  const owner = node.closest<HTMLElement>('[data-job], [data-action]');
  return [
    owner?.dataset.job ?? owner?.dataset.action ?? '',
    node.tagName,
    node.getAttribute('id'),
    node.getAttribute('name'),
    node.getAttribute('href'),
    node instanceof HTMLDetailsElement
      ? node.querySelector('summary')?.textContent
      : node.textContent?.slice(0, 180),
  ].join('|');
}
export function preserveView(root: HTMLElement, update: () => void): void {
  const opened = new Set([...root.querySelectorAll('details[open]')].map(key));
  const focused = document.activeElement;
  const focusKey = focused && root.contains(focused) ? key(focused) : null;
  const scroll = [...root.querySelectorAll<HTMLElement>('pre, .table-region, .graph-region')].map(
    (node) => ({ key: key(node), top: node.scrollTop, left: node.scrollLeft }),
  );
  const x = window.scrollX,
    y = window.scrollY;
  update();
  for (const node of root.querySelectorAll('details')) node.open = opened.has(key(node));
  for (const node of root.querySelectorAll<HTMLElement>('pre, .table-region, .graph-region')) {
    const position = scroll.find((item) => item.key === key(node));
    if (position) {
      node.scrollTop = position.top;
      node.scrollLeft = position.left;
    }
  }
  if (focusKey)
    [...root.querySelectorAll<HTMLElement>('button, a, input, select, summary, main, [data-job]')]
      .find((node) => key(node) === focusKey)
      ?.focus({ preventScroll: true });
  window.scrollTo(x, y);
}
