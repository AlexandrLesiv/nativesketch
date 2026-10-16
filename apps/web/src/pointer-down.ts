import type { EditorHost } from './host.ts';
import { el } from './elements.ts';
import { floats } from './engine.ts';
export function installPointerDown(app: EditorHost) {
  app.canvas.addEventListener('pointerdown', (e) =>
    app.safe(() => {
      if (e.button !== 0 && e.button !== 1) return;
      app.canvas.focus();
      const p = app.position(e);
      app.contacts.set(e.pointerId, p);
      app.canvas.setPointerCapture(e.pointerId);
      if (app.contacts.size >= 2) {
        app.cancel();
        if (app.pair.length === 0) {
          app.pair = [...app.contacts.keys()].slice(0, 2);
          app.navigation = app.metric();
        }
        return;
      }
      if (app.pair.length) return;
      app.owner = e.pointerId;
      if (app.space || e.button === 1) {
        app.mode = 'pan';
        return;
      }
      const q = app.point(p.x, p.y),
        s = app.engine.state();
      if (app.tool === 'select') {
        if (s.selected) {
          const [x, y, w, h] = s.bounds;
          const corners = [
            [x, y],
            [x + w, y],
            [x, y + h],
            [x + w, y + h],
          ];
          const index = corners.findIndex(([a, b]) => Math.hypot(q.x - a, q.y - b) < 14 / app.zoom);
          if (index >= 0) {
            const [a, b] = corners[3 - index];
            app.resize = {
              x: a,
              y: b,
              distance: Math.max(0.01, Math.hypot(q.x - a, q.y - b)),
            };
            app.engine.command(11);
            app.mode = 'resize';
            return;
          }
        }
        app.engine.command(7, floats(q.x, q.y, 8 / app.zoom));
        app.mode = 'select';
        return;
      }
      const hex = el<HTMLInputElement>('color').value,
        rgba = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
      rgba.push(1);
      app.engine.begin(
        BigInt(e.pointerId),
        e.pointerType === 'pen' ? 1 : e.pointerType === 'touch' ? 2 : 0,
        rgba,
        Number(el<HTMLInputElement>('width').value),
      );
      app.mode = 'ink';
      app.sample = 0n;
      app.append(e);
    }),
  );
}
