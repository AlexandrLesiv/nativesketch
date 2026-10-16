import type { EditorHost } from './host.ts';
import { el } from './elements.ts';
import { importFile, resetReplacementChoice } from './engine.ts';
export function download(app: EditorHost) {
  app.cancel();
  const snapshot = app.engine.snapshot();
  const url = URL.createObjectURL(
    new Blob([snapshot.bytes.slice().buffer], {
      type: 'application/octet-stream',
    }),
  );
  const a = document.createElement('a');
  a.href = url;
  a.download = 'Drawing.nsketch';
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 60000);
  app.refresh();
}
export async function guard(app: EditorHost) {
  app.cancel();
  if (!app.engine.state().dirty) return true;
  const dialog = el<HTMLDialogElement>('guard');
  resetReplacementChoice(dialog);
  dialog.showModal();
  const result = await new Promise<string>((resolve) =>
    dialog.addEventListener('close', () => resolve(dialog.returnValue), {
      once: true,
    }),
  );
  if (result === 'download') app.download();
  return result === 'discard';
}

export function installFiles(app: EditorHost) {
  el<HTMLDialogElement>('guard').addEventListener('cancel', () =>
    resetReplacementChoice(el<HTMLDialogElement>('guard')),
  );
  app.click('save', app.download);
  app.click('new', () => {
    void app
      .guard()
      .then((ok) => {
        if (ok) app.safe(() => app.engine.command(16));
      })
      .catch(app.error);
  });
  app.click('open', () => {
    void app
      .guard()
      .then((ok) => {
        if (ok) el<HTMLInputElement>('file').click();
      })
      .catch(app.error);
  });
  el<HTMLInputElement>('file').addEventListener('change', async (e) => {
    const input = e.target as HTMLInputElement;
    const f = input.files?.[0];
    input.value = '';
    if (!f) return;
    try {
      await importFile(app.engine, f, app.guard);
      app.refresh();
    } catch (e) {
      app.error(e);
    }
  });
}
