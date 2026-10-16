import type { EditorHost } from './host.ts';

export function paint(app: EditorHost, s: ReturnType<EditorHost['engine']['state']>) {
  const frame = app.scene.update([
    -app.tx / app.zoom,
    -app.ty / app.zoom,
    app.width / app.zoom,
    app.height / app.zoom,
    32 / app.zoom,
  ]);
  const dpr = window.devicePixelRatio || 1;
  app.ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  app.ctx.fillStyle = '#fff';
  app.ctx.fillRect(0, 0, app.width, app.height);
  app.ctx.translate(app.tx, app.ty);
  app.ctx.scale(app.zoom, app.zoom);
  for (const stroke of frame.strokes) {
    const geometry = frame.geometry.get(stroke.geometry)!;
    app.ctx.save();
    app.ctx.translate(stroke.tx, stroke.ty);
    app.ctx.scale(stroke.scale, stroke.scale);
    app.ctx.translate(geometry.origin[0], geometry.origin[1]);
    app.ctx.beginPath();
    const d = new DataView(geometry.commands.buffer);
    for (let o = 0; o < geometry.commands.length; o += 88) {
      const c = [8, 16, 24, 32].map((n) => d.getFloat64(o + n, true));
      app.ctx.fillStyle = `rgba(${c[0] * 255},${c[1] * 255},${c[2] * 255},${c[3]})`;
      const p = [40, 48, 56, 64, 72, 80].map((n) => d.getFloat64(o + n, true));
      switch (d.getUint32(o, true)) {
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
          break;
      }
    }
    app.ctx.fill('nonzero');
    app.ctx.restore();
  }
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
