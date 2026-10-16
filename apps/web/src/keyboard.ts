import type { EditorHost } from './host.ts';
import { el } from './elements.ts';
export function installKeyboard(app: EditorHost) {
  window.addEventListener('beforeunload', (e) => {
    if (app.protectFailedDocument || app.lastDirty) {
      e.preventDefault();
      e.returnValue = '';
    }
  });
  document.addEventListener('keydown', (e) => {
    if ((e.target as HTMLElement).closest('input,textarea,select,dialog,[contenteditable=true]'))
      return;
    if (!app.engine?.isAvailable() || app.failed) return;
    const canvasFocused = e.target === app.canvas;
    const modifier = e.ctrlKey || e.metaKey;
    const key = e.key.toLowerCase();
    if (canvasFocused && key === ' ') {
      app.space = true;
      e.preventDefault();
      return;
    }
    let action: (() => void) | undefined;
    if (key === 'escape') action = app.cancel;
    else if (modifier && key === 'z')
      action = () => {
        const s = app.engine.state();
        if (e.shiftKey ? s.redo : s.undo) app.engine.command(e.shiftKey ? 15 : 14);
      };
    else if (modifier && key === 's') action = app.download;
    else if (modifier && key === 'o') action = () => el('open').click();
    else if (modifier && key === 'n') action = () => el('new').click();
    else if (canvasFocused && !modifier && key === 'p') action = () => app.chooseTool('pen');
    else if (canvasFocused && !modifier && key === 'v') action = () => app.chooseTool('select');
    else if (
      canvasFocused &&
      (key === 'delete' || key === 'backspace') &&
      app.engine.state().selected
    )
      action = () => app.engine.command(10);
    if (action) {
      e.preventDefault();
      app.safe(() => {
        app.cancel();
        action!();
      });
    }
  });
  document.addEventListener('keyup', (e) => {
    if (e.key === ' ') app.space = false;
  });
}
