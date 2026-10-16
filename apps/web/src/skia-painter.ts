import type { CanvasKit, Surface, Paint } from 'canvaskit-wasm';
import type { Frame } from './scene.ts';
import { SkiaPaths } from './skia-paths.ts';
export interface Camera {
  x: number;
  y: number;
  zoom: number;
  dpr: number;
}
export class SkiaPainter {
  readonly paths: SkiaPaths;
  private kit: CanvasKit;
  private fill: Paint;
  private selection: Paint;
  constructor(kit: CanvasKit, paths = new SkiaPaths(kit)) {
    this.kit = kit;
    this.paths = paths;
    this.fill = new kit.Paint();
    this.fill.setAntiAlias(true);
    this.fill.setStyle(kit.PaintStyle.Fill);
    this.selection = new kit.Paint();
    this.selection.setAntiAlias(true);
    this.selection.setStyle(kit.PaintStyle.Stroke);
    this.selection.setColor(kit.Color4f(23 / 255, 97 / 255, 219 / 255, 1));
  }
  draw(surface: Surface, frame: Frame, camera: Camera, bounds?: number[]) {
    const stage = this.paths.stage(frame, surface.width(), surface.height());
    try {
      const canvas = surface.getCanvas(),
        factorX = surface.width() / frame.viewport[2],
        factorY = surface.height() / frame.viewport[3];
      canvas.clear(this.kit.WHITE);
      for (const stroke of frame.strokes) {
        const entry = stage.paths.get(stroke)!,
          geometry = frame.geometry.get(stroke.geometry)!;
        const x = (geometry.origin[0] * stroke.scale + stroke.tx - frame.viewport[0]) * factorX;
        const y = (geometry.origin[1] * stroke.scale + stroke.ty - frame.viewport[1]) * factorY;
        const scaleX = stroke.scale * factorX,
          scaleY = stroke.scale * factorY;
        canvas.save();
        try {
          if (!entry.pixels) canvas.concat([scaleX, 0, x, 0, scaleY, y, 0, 0, 1]);
          this.fill.setColor(
            this.kit.Color4f(entry.color[0], entry.color[1], entry.color[2], entry.color[3]),
          );
          canvas.drawPath(entry.path, this.fill);
        } finally {
          canvas.restore();
        }
      }
      if (bounds) {
        const [x, y, w, h] = bounds;
        const left = (x - frame.viewport[0]) * factorX,
          top = (y - frame.viewport[1]) * factorY;
        const right = left + w * factorX,
          bottom = top + h * factorY;
        this.selection.setStrokeWidth(camera.dpr);
        canvas.drawRect([left, top, right, bottom], this.selection);
        this.fill.setColor(this.kit.WHITE);
        const half = 4 * camera.dpr;
        for (const [a, b] of [
          [left, top],
          [right, top],
          [left, bottom],
          [right, bottom],
        ]) {
          const rect = [a - half, b - half, a + half, b + half];
          canvas.drawRect(rect, this.fill);
          canvas.drawRect(rect, this.selection);
        }
      }
      surface.flush();
      stage.commit();
    } finally {
      stage.abort();
    }
  }
  dispose() {
    this.paths.dispose();
    this.fill.delete();
    this.selection.delete();
  }
}
