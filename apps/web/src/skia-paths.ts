import type { CanvasKit, Path } from 'canvaskit-wasm';
import type { Frame, Geometry, Stroke } from './scene.ts';
import { f64, u32 } from './wire.ts';
export interface CachedPath {
  path: Path;
  pixels: boolean;
  color: number[];
  bytes: number;
}
/** Current-frame admission: 4096 paths/32 MiB estimate; staging peak is at most twice each bound. */
export class SkiaPaths {
  private kit: CanvasKit;
  private entries = new Map<string, CachedPath>();
  private decisions = new Map<string, string>();
  readonly maximumBytes: number;
  readonly maximumCount: number;
  owned = 0;
  constructor(kit: CanvasKit, maximumBytes = 32 * 1024 * 1024, maximumCount = 4096) {
    this.kit = kit;
    this.maximumBytes = maximumBytes;
    this.maximumCount = maximumCount;
  }
  private build(g: Geometry, color: number[], pixels: boolean): CachedPath {
    const builder = new this.kit.PathBuilder();
    let consumed = false;
    try {
      builder.setFillType(this.kit.FillType.Winding);
      for (let offset = 0; offset < g.commands.length; offset += 88) {
        const p = [40, 48, 56, 64, 72, 80].map(
          (n, i) => f64(g.commands, offset + n) - g.origin[i % 2],
        );
        switch (u32(g.commands, offset)) {
          case 0:
            builder.moveTo(p[0], p[1]);
            break;
          case 1:
            builder.lineTo(p[0], p[1]);
            break;
          case 2:
            builder.cubicTo(p[0], p[1], p[2], p[3], p[4], p[5]);
            break;
          case 3:
            builder.close();
            break;
          default:
            throw Error('Unsupported geometry verb');
        }
      }
      const path = builder.detachAndDelete();
      consumed = true;
      this.owned++;
      return {
        path,
        color,
        pixels,
        bytes: g.commands.length + 128,
      };
    } finally {
      if (!consumed) builder.delete();
    }
  }
  stage(frame: Frame, width: number, height: number) {
    if (frame.strokes.length > this.maximumCount)
      throw Error('Visible drawing exceeds the renderer cache capacity');
    const next = new Map<string, CachedPath>(),
      decisions = new Map<string, string>();
    const paths = new Map<Stroke, CachedPath>(),
      created: CachedPath[] = [];
    let bytes = 0;
    try {
      for (const stroke of frame.strokes) {
        const geometry = frame.geometry.get(stroke.geometry)!;
        const decision = JSON.stringify([
          String(stroke.geometry),
          stroke.scale,
          stroke.tx,
          stroke.ty,
          frame.viewport,
          width,
          height,
        ]);
        let key = this.decisions.get(decision);
        let entry = key ? this.entries.get(key) : undefined;
        if (!entry) {
          if (!frame.prepare) throw Error('Missing leased scene preparation');
          const prepared = frame.prepare(stroke, width, height, this.maximumBytes - bytes - 128);
          key = prepared ? decision : `local:${stroke.geometry}`;
          entry = next.get(key) ?? this.entries.get(key);
          if (!entry) {
            const g = prepared ?? geometry;
            if (bytes + g.commands.length + 128 > this.maximumBytes)
              throw Error('Visible drawing exceeds the renderer cache capacity');
            entry = this.build(
              g,
              [8, 16, 24, 32].map((o) => f64(geometry.commands, o)),
              prepared !== null,
            );
            created.push(entry);
          }
        }
        if (!key) throw Error('Missing prepared path cache key');
        if (!next.has(key)) bytes += entry.bytes;
        if (bytes > this.maximumBytes)
          throw Error('Visible drawing exceeds the renderer cache capacity');
        next.set(key, entry);
        decisions.set(decision, key);
        paths.set(stroke, entry);
      }
    } catch (error) {
      for (const entry of created) this.remove(entry);
      throw error;
    }
    let finished = false;
    return {
      paths,
      commit: () => {
        if (finished) throw Error('Renderer stage already completed');
        finished = true;
        for (const [id, entry] of this.entries) if (!next.has(id)) this.remove(entry);
        this.entries = next;
        this.decisions = decisions;
      },
      abort: () => {
        if (finished) return;
        finished = true;
        for (const entry of created) this.remove(entry);
      },
    };
  }
  private remove(entry: CachedPath) {
    entry.path.delete();
    this.owned--;
  }
  dispose() {
    for (const entry of this.entries.values()) this.remove(entry);
    this.entries.clear();
    this.decisions.clear();
  }
}
