import type { EditorHost } from './host.ts';
function finish(app: EditorHost, e: PointerEvent, aborted: boolean) {
  app.safe(() => {
    if (app.owner === e.pointerId) {
      if (aborted) app.cancel();
      else {
        if (app.mode === 'ink') {
          app.append(e);
          app.engine.commit(BigInt(e.pointerId));
        }
        if (app.mode === 'resize' && app.resize) {
          const p = app.position(e),
            q = app.point(p.x, p.y);
          app.engine.transform(
            Math.max(
              0.01,
              Math.hypot(q.x - app.resize.x, q.y - app.resize.y) / app.resize.distance,
            ),
            app.resize.x,
            app.resize.y,
          );
          app.engine.commitTransform();
        }
        app.owner = undefined;
        app.mode = '';
        app.resize = undefined;
      }
    }
    app.contacts.delete(e.pointerId);
    if (app.pair.includes(e.pointerId)) app.navigation = undefined;
    if (!app.contacts.size) {
      app.pair = [];
      app.navigation = undefined;
    }
  });
}
export function installPointerFinish(app: EditorHost) {
  app.canvas.addEventListener('pointerup', (e) => finish(app, e, false));
  app.canvas.addEventListener('pointercancel', (e) => finish(app, e, true));
  app.canvas.addEventListener('lostpointercapture', (e) => {
    if (app.contacts.has(e.pointerId)) finish(app, e, true);
  });
}
