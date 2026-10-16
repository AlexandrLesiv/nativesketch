import type { Frame } from './scene.ts';
import type { Camera } from './skia-painter.ts';
import { f64, u32 } from './wire.ts';
/** Match native's safe device extent; shared f64 viewport clipping is still pending here. */
export function requireSafePrecision(frame: Frame, camera: Camera) {
  const factor = camera.zoom * camera.dpr;
  for (const stroke of frame.strokes) {
    const geometry = frame.geometry.get(stroke.geometry)!;
    for (let offset = 0; offset < geometry.commands.length; offset += 88) {
      const verb = u32(geometry.commands, offset),
        count = verb === 2 ? 3 : verb < 2 ? 1 : 0;
      for (let point = 0; point < count; point++) {
        const x =
          (f64(geometry.commands, offset + 40 + point * 16) * stroke.scale + stroke.tx - camera.x) *
          factor;
        const y =
          (f64(geometry.commands, offset + 48 + point * 16) * stroke.scale + stroke.ty - camera.y) *
          factor;
        if (Math.abs(x) > 4096 || Math.abs(y) > 4096)
          throw Error('Drawing needs viewport precision preparation; download remains available');
      }
    }
  }
}
