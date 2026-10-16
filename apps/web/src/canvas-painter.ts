import type { EditorHost } from './host.ts';

export function paint(app: EditorHost, s: ReturnType<EditorHost['engine']['state']>) {
  const dpr = window.devicePixelRatio || 1;
  app.ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  app.ctx.fillStyle = '#fff';
  app.ctx.fillRect(0, 0, app.width, app.height);
  app.ctx.translate(app.tx, app.ty);
  app.ctx.scale(app.zoom, app.zoom);
  const bytes = app.engine.command(6),
    d = new DataView(bytes.buffer);
  let id: bigint | undefined;
  const flush = () => {
    if (id !== undefined) app.ctx.fill('nonzero');
  };
  for (let o = 0; o < bytes.length; o += 96) {
    const next = d.getBigUint64(o, true);
    if (next !== id) {
      flush();
      app.ctx.beginPath();
      id = next;
      const c = [16, 24, 32, 40].map((n) => d.getFloat64(o + n, true));
      app.ctx.fillStyle = `rgba(${c[0] * 255},${c[1] * 255},${c[2] * 255},${c[3]})`;
    }
    const p = [48, 56, 64, 72, 80, 88].map((n) => d.getFloat64(o + n, true));
    switch (d.getUint32(o + 8, true)) {
      case 0:
        app.ctx.moveTo(p[0], p[1]);
        break;
      case 1:
        app.ctx.lineTo(p[0], p[1]);
        break;
      case 2:
        app.ctx.bezierCurveTo(p[0], p[1], p[2], p[3], p[4], p[5]);
        break;
      case 3:
        app.ctx.closePath();
    }
  }
  flush();
  if (s.selected) {
    const [x, y, w, h] = s.bounds;
    app.ctx.strokeStyle = '#1761db';
    app.ctx.lineWidth = 1 / app.zoom;
    app.ctx.strokeRect(x, y, w, h);
    for (const [a, b] of [
      [x, y],
      [x + w, y],
      [x, y + h],
      [x + w, y + h],
    ]) {
      app.ctx.fillStyle = '#fff';
      app.ctx.fillRect(a - 4 / app.zoom, b - 4 / app.zoom, 8 / app.zoom, 8 / app.zoom);
      app.ctx.strokeRect(a - 4 / app.zoom, b - 4 / app.zoom, 8 / app.zoom, 8 / app.zoom);
    }
  }
}
