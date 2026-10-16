import type { EditorHost } from './host.ts';
import { el } from './elements.ts';
import { resetReplacementChoice } from './engine.ts';
import { FileWork } from './file-work.ts';
import { u32, u64 } from './wire.ts';
const hosts = new WeakMap<EditorHost, BrowserFiles>();
class BrowserFiles {
  private app: EditorHost;
  readonly work: FileWork;
  private picker = 0n;
  private confirmation = 0n;
  private urls = new Set<string>();
  constructor(app: EditorHost) {
    this.app = app;
    this.work = new FileWork(app.engine);
  }
  dispatch = (bytes: Uint8Array) => {
    if (this.urls.size >= 8) throw Error('Too many downloads are pending');
    const url = URL.createObjectURL(
      new Blob([bytes.slice().buffer], { type: 'application/octet-stream' }),
    );
    this.urls.add(url);
    try {
      const a = document.createElement('a');
      a.href = url;
      a.download = 'Drawing.nsketch';
      a.click();
    } finally {
      setTimeout(() => {
        URL.revokeObjectURL(url);
        this.urls.delete(url);
      }, 1000);
    }
  };
  effect = (response: Uint8Array) => {
    const kind = u32(response, 232),
      token = u64(response, 240);
    if (kind === 1) this.effect(this.work.supply(token));
    if (kind === 2) {
      this.picker = token;
      const input = el<HTMLInputElement>('file');
      input.value = '';
      input.click();
    }
    if (kind === 3) {
      this.confirmation = token;
      const dialog = el<HTMLDialogElement>('guard');
      resetReplacementChoice(dialog);
      dialog.showModal();
    }
    if (kind === 6) this.effect(this.app.engine.reply(token, 4));
    if (kind === 9) throw Error('The file operation could not be completed.');
    void this.work.drain(this.effect, this.dispatch, this.app.error);
    this.app.refresh();
  };
  intent(tag: number) {
    this.effect(this.app.engine.file(tag));
  }
  choice(value: string) {
    const token = this.confirmation;
    this.confirmation = 0n;
    if (!token) return;
    this.effect(this.app.engine.reply(token, value === 'discard' ? 3 : 1));
    if (value === 'download') this.intent(3);
  }
  selected(source?: File) {
    const token = this.picker;
    this.picker = 0n;
    if (!token) return;
    this.effect(source ? this.work.supply(token, source) : this.app.engine.reply(token, 1));
    this.app.canvas.focus();
  }
}
function files(app: EditorHost) {
  let value = hosts.get(app);
  if (!value) {
    value = new BrowserFiles(app);
    hosts.set(app, value);
  }
  return value;
}
export function download(app: EditorHost) {
  files(app).intent(3);
}
export function installFiles(app: EditorHost) {
  const dialog = el<HTMLDialogElement>('guard');
  dialog.addEventListener('cancel', () => resetReplacementChoice(dialog));
  // Handle the trusted click before dialog close: source picking keeps user activation.
  dialog.addEventListener('click', (event) => {
    const button = (event.target as HTMLElement).closest('button');
    if (!button) return;
    event.preventDefault();
    dialog.close(button.value);
    app.safe(() => files(app).choice(button.value));
  });
  dialog.addEventListener('close', () => app.safe(() => files(app).choice(dialog.returnValue)));
  app.click('save', app.download);
  app.click('new', () => files(app).intent(4));
  app.click('open', () => files(app).intent(5));
  const input = el<HTMLInputElement>('file');
  input.addEventListener('cancel', () => app.safe(() => files(app).selected()));
  input.addEventListener('change', () => {
    const source = input.files?.[0];
    input.value = '';
    app.safe(() => files(app).selected(source));
  });
}
