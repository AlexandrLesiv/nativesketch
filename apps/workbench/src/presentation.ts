import type { Diagnostic, Section, Source } from '../../../scripts/workbench/contracts/index.ts';
import { detail, el, notice, textList } from './dom.ts';
export function identity(source: Source): HTMLElement {
  return detail(
    'Full source identity',
    el('p', `Revision: ${source.revision}`),
    el('p', `Dirty diff SHA-256: ${source.dirtyDiffSha256 ?? 'Clean'}`),
  );
}
export function diagnostics(items: Diagnostic[]): HTMLElement {
  return textList(
    items.map(
      (item) =>
        `${item.severity}: ${item.code}${item.path ? ` (${item.path})` : ''} — ${item.message}`,
    ),
  );
}
export function sectionState(
  section: Pick<Section<unknown>, 'state' | 'diagnostics'>,
): HTMLElement {
  const node = el('div');
  if (section.state !== 'ready')
    node.append(notice(`Section state: ${section.state}. This is not a successful empty result.`));
  node.append(diagnostics(section.diagnostics));
  return node;
}
export function sourcePaths(paths: string[]): HTMLElement {
  return textList(paths.length ? paths : ['No evidence path recorded']);
}
export function jsonDetail(label: string, value: unknown): HTMLElement {
  return detail(label, el('pre', JSON.stringify(value, null, 2)));
}
