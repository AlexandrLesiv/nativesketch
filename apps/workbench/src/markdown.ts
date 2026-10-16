import type { Document } from '../../../scripts/workbench/contracts/index.ts';
import { headingSlug } from './headings.ts';
import { el } from './dom.ts';
import { link } from './route.ts';
function resolvePath(base: string, target: string): string | null {
  if (/^[a-z][a-z\d+.-]*:/i.test(target) || target.startsWith('/') || target.includes('\\'))
    return null;
  const parts = base.split('/').slice(0, -1);
  for (const part of target.split('/')) {
    if (part === '..') {
      if (!parts.length) return null;
      parts.pop();
    } else if (part && part !== '.') parts.push(part);
  }
  return parts.join('/');
}
export function documentLink(
  label: string,
  target: string,
  current: Document,
  docs: Document[],
): HTMLElement {
  if (/^https?:\/\//i.test(target)) {
    try {
      const url = new URL(target);
      if (url.username || url.password) return el('span', `${label} (unsupported link)`);
      const anchor = el('a', `${label} ↗ (external)`);
      anchor.href = url.href;
      anchor.rel = 'noopener noreferrer';
      return anchor;
    } catch {
      return el('span', `${label} (invalid link)`);
    }
  }
  const [path, heading] = target.split('#');
  const resolved = path ? resolvePath(current.id, path) : current.id;
  const found = docs.find((doc) => doc.id === resolved);
  return found
    ? link(label, 'docs', { selected: found.id, ...(heading ? { heading } : {}) })
    : el('span', `${label} (Not in this snapshot)`);
}
function inline(node: HTMLElement, text: string, current: Document, docs: Document[]): void {
  const pattern = /(!?)\[([^\]]*)\]\(([^\s)]+)\)/g;
  let offset = 0;
  for (const match of text.matchAll(pattern)) {
    node.append(text.slice(offset, match.index));
    node.append(
      match[1]
        ? el('span', `[Unsupported image: ${match[2]}]`)
        : documentLink(match[2] ?? '', match[3] ?? '', current, docs),
    );
    offset = match.index + match[0].length;
  }
  node.append(text.slice(offset));
}
export function markdown(current: Document, docs: Document[]): HTMLElement {
  const article = el('article');
  if (current.format === 'text') {
    article.append(el('pre', current.text));
    return article;
  }
  if (current.text.length > 262_144 || current.text.split('\n', 5001).length > 5000) {
    article.append(
      el(
        'p',
        'Large document: rendered as complete plain text to keep navigation responsive. Search still covers the full document.',
      ),
      el('pre', current.text),
    );
    return article;
  }
  const outline = el('nav');
  outline.setAttribute('aria-label', 'Document headings');
  const content = el('div');
  let fenced: string[] | null = null;
  const headingCounts = new Map<string, number>();
  for (const line of current.text.split('\n')) {
    if (line.startsWith('```')) {
      if (fenced) {
        content.append(el('pre', fenced.join('\n')));
        fenced = null;
      } else fenced = [];
      continue;
    }
    if (fenced) {
      fenced.push(line);
      continue;
    }
    const heading = /^(#{1,6})\s+(.+)$/.exec(line);
    if (heading) {
      const title = heading[2] ?? '';
      const node = el('h3', title);
      node.id = `doc-heading-${headingSlug(title, headingCounts)}`;
      const anchor = el('a', title);
      anchor.href = `#${new URLSearchParams({ view: 'docs', selected: current.id, heading: node.id.slice('doc-heading-'.length) })}`;
      outline.append(anchor, el('br'));
      content.append(node);
    } else if (line.trim()) {
      const node = el('p');
      inline(node, line, current, docs);
      content.append(node);
    }
  }
  if (fenced) content.append(el('pre', fenced.join('\n')));
  article.append(outline, content);
  return article;
}
