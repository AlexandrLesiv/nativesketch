import type { EditorHost } from './host.ts';
import { el } from './elements.ts';
import { floats } from './engine.ts';
export function chooseTool(app: EditorHost, next: string) {
  app.cancel();
  app.tool = next;
  if (app.tool === 'pen') app.engine.command(9);
  el('pen').setAttribute('aria-pressed', String(app.tool === 'pen'));
  el('select').setAttribute('aria-pressed', String(app.tool === 'select'));
  app.refresh();
}

export function click(app: EditorHost, id: string, fn: () => void) {
  el(id).addEventListener('click', () =>
    app.safe(() => {
      app.cancel();
      fn();
    }),
  );
}
export function installControls(app: EditorHost) {
  app.click('pen', () => app.chooseTool('pen'));
  app.click('select', () => app.chooseTool('select'));
  for (const [id, op] of [
    ['undo', 14],
    ['redo', 15],
    ['delete', 10],
  ] as const)
    app.click(id, () => app.engine.command(op));
  app.click('previous', () => {
    app.chooseTool('select');
    app.engine.cycle(-1);
  });
  app.click('next', () => {
    app.chooseTool('select');
    app.engine.cycle(1);
  });
  app.click('apply', () => {
    const [x, y, w, h] = app.engine.state().bounds;
    app.engine.command(11);
    try {
      app.engine.command(
        12,
        floats(Number(el<HTMLInputElement>('scale').value) / 100, x + w / 2, y + h / 2),
      );
      app.engine.command(13);
    } catch (e) {
      app.cancel();
      throw e;
    }
  });
  app.click('in', () => app.setZoom(app.zoom * 1.25));
  app.click('out', () => app.setZoom(app.zoom / 1.25));
  app.click('reset', () => app.setZoom(1));
}
