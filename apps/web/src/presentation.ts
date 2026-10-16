import type { EditorHost } from './host.ts';
import { el } from './elements.ts';
import { paint } from './canvas-painter.ts';
export function disableEngine(app: EditorHost, message: string, protectDocument: boolean) {
  app.failed = true;
  app.protectFailedDocument = protectDocument;
  document
    .querySelectorAll<HTMLButtonElement | HTMLInputElement>(
      'header button,.tools button,.tools input',
    )
    .forEach((control) => (control.disabled = true));
  app.canvas.setAttribute('aria-disabled', 'true');
  el('status').textContent = message;
  el('reload').hidden = false;
}

export function error(app: EditorHost, e: unknown) {
  el('error-text').textContent = String(e);
  el<HTMLDialogElement>('error').showModal();
}
export function safe(app: EditorHost, fn: () => void) {
  if (!app.engine || app.failed) return;
  try {
    fn();
    app.refresh();
  } catch (e) {
    if (!app.engine.isAvailable()) {
      app.disableEngine(
        'Drawing engine stopped. Reload to recover; unsaved work may be lost.',
        true,
      );
      app.error(
        'The drawing engine stopped unexpectedly. Reload this page to recover. Unsaved changes cannot be recovered from this failed engine.',
      );
    } else app.error(e);
  }
}
export function refresh(app: EditorHost) {
  if (!app.engine?.isAvailable()) return;
  const s = app.engine.state();
  app.lastDirty = s.dirty;
  app.tool = s.tool;
  for (const id of ['pen', 'select']) el(id).setAttribute('aria-pressed', String(id === s.tool));
  if (!s.interaction && (app.mode === 'ink' || app.mode === 'resize')) {
    const owner = app.owner;
    app.owner = undefined;
    if (owner !== undefined && app.canvas.hasPointerCapture(owner))
      app.canvas.releasePointerCapture(owner);
    app.mode = '';
    app.resize = undefined;
  }
  for (const [id, disabled] of Object.entries({
    undo: !s.undo,
    redo: !s.redo,
    previous: !s.count,
    next: !s.count,
    scale: !s.selected,
    apply: !s.selected,
    delete: !s.selected,
  }))
    (el(id) as HTMLButtonElement).disabled = disabled;
  el('status').textContent =
    `${s.count} stroke${s.count === 1n ? '' : 's'} · ${Math.round(app.zoom * 100)}%${s.dirty ? ' · Unsaved' : ''}${s.selected ? ' · Stroke selected' : ''}`;
  app.canvas.setAttribute(
    'aria-label',
    `Drawing canvas, ${s.count} strokes, ${s.selected ? 'stroke selected, ' : ''}${Math.round(app.zoom * 100)} percent zoom`,
  );
  el('hint').hidden = s.count > 0n || s.interaction !== 0;
  if (!app.repaint)
    app.repaint = requestAnimationFrame(() => {
      app.repaint = 0;
      try {
        paint(app, app.engine.state());
      } catch (error) {
        app.error(error);
      }
    });
}
