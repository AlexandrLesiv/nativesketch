import type { EditorHost } from './host.ts';
import { el } from './elements.ts';
export function chooseTool(app: EditorHost, next: string) {
  app.engine.action(next === 'pen' ? 'pen' : 'select');
  app.refresh();
}
export function click(app: EditorHost, id: string, fn: () => void) {
  el(id).addEventListener('click', () => app.safe(fn));
}
export function installControls(app: EditorHost) {
  for (const id of ['pen', 'select', 'undo', 'redo', 'delete', 'previous', 'next'] as const)
    app.click(id, () => app.engine.action(id));
  app.click('apply', () => app.engine.scale(Number(el<HTMLInputElement>('scale').value) / 100));
  const brush = () => {
    const width = el<HTMLInputElement>('width');
    if (
      !width.value ||
      !Number.isFinite(width.valueAsNumber) ||
      width.valueAsNumber < 0.1 ||
      width.valueAsNumber > 100
    )
      return;
    const hex = el<HTMLInputElement>('color').value;
    const rgba = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
    app.engine.brush([...rgba, 1], Number(el<HTMLInputElement>('width').value));
  };
  for (const id of ['color', 'width']) el(id).addEventListener('change', () => app.safe(brush));
  app.click('in', () => app.setZoom(app.zoom * 1.25));
  app.click('out', () => app.setZoom(app.zoom / 1.25));
  app.click('reset', () => app.setZoom(1));
}
