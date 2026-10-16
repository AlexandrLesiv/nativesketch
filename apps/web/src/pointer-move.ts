import type { EditorHost } from './host.ts';
export function installPointerMove(app: EditorHost) {
  app.canvas.addEventListener('pointermove', (e) =>
    app.safe(() => {
      if (!app.contacts.has(e.pointerId)) return;
      const old = app.contacts.get(e.pointerId)!;
      const p = app.position(e);
      app.contacts.set(e.pointerId, p);
      if (app.pair.length) {
        const next = app.metric();
        if (next && app.navigation) {
          const q = app.point(app.navigation.x, app.navigation.y);
          app.zoom = Math.max(
            0.05,
            Math.min(32, (app.zoom * next.distance) / app.navigation.distance),
          );
          app.tx = next.x - q.x * app.zoom;
          app.ty = next.y - q.y * app.zoom;
        }
        app.navigation = next;
        return;
      }
      if (app.owner !== e.pointerId) return;
      if (app.mode === 'pan') {
        app.tx += p.x - old.x;
        app.ty += p.y - old.y;
      } else if (app.mode === 'ink') app.append(e);
      else if (app.mode === 'resize' && app.resize) {
        const q = app.point(p.x, p.y);
        app.engine.transform(
          Math.max(0.01, Math.hypot(q.x - app.resize.x, q.y - app.resize.y) / app.resize.distance),
          app.resize.x,
          app.resize.y,
        );
      }
    }),
  );
}
