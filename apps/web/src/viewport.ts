import type { EditorHost } from './host.ts';

export function setZoom(app: EditorHost, value: number, x = app.width / 2, y = app.height / 2) {
  app.cancel();
  const p = app.point(x, y);
  app.zoom = Math.max(0.05, Math.min(32, value));
  app.tx = x - p.x * app.zoom;
  app.ty = y - p.y * app.zoom;
  app.refresh();
}

export function installViewport(app: EditorHost) {
  app.canvas.addEventListener(
    'wheel',
    (e) => {
      e.preventDefault();
      app.safe(() => {
        app.cancel();
        if (e.ctrlKey || e.metaKey) {
          const r = app.canvas.getBoundingClientRect();
          app.setZoom(app.zoom * Math.exp(-e.deltaY * 0.01), e.clientX - r.left, e.clientY - r.top);
        } else {
          app.tx -= e.deltaX;
          app.ty -= e.deltaY;
        }
      });
    },
    { passive: false },
  );
  window.addEventListener('blur', () => {
    app.space = false;
    app.cancel();
    app.contacts.clear();
    app.pair = [];
    app.navigation = undefined;
    app.refresh();
  });
  const resize = () => {
    const r = app.canvas.getBoundingClientRect();
    if (r.width === app.width && r.height === app.height) return;
    app.cancel();
    app.tx += (r.width - app.width) / 2;
    app.ty += (r.height - app.height) / 2;
    if (!app.width) {
      app.tx = 0;
      app.ty = 0;
    }
    app.width = r.width;
    app.height = r.height;
    const dpr = window.devicePixelRatio || 1;
    app.canvas.width = Math.round(app.width * dpr);
    app.canvas.height = Math.round(app.height * dpr);
    app.refresh();
  };
  resize();
  new ResizeObserver(resize).observe(app.canvas);
}
