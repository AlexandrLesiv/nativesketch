import type { CanvasKit, Path } from 'canvaskit-wasm';
import type { Frame, Geometry } from './scene.ts';
import { f64, u32 } from './wire.ts';
export interface CachedPath {
  path: Path;
  color: number[];
  bytes: number;
}
/** Current-frame admission: 4096 paths/32 MiB estimate; staging peak is at most twice each bound. */
export class SkiaPaths {
  private kit: CanvasKit;
  private entries = new Map<bigint, CachedPath>();
  readonly maximumBytes: number;
  readonly maximumCount: number;
  owned = 0;
  constructor(kit: CanvasKit, maximumBytes = 32 * 1024 * 1024, maximumCount = 4096) {
    this.kit = kit;
    this.maximumBytes = maximumBytes;
    this.maximumCount = maximumCount;
  }
  private build(g: Geometry): CachedPath {
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
        color: [8, 16, 24, 32].map((o) => f64(g.commands, o)),
        bytes: g.commands.length + 128,
      };
    } finally {
      if (!consumed) builder.delete();
    }
  }
  stage(frame: Frame) {
    let bytes = 0;
    for (const geometry of frame.geometry.values()) bytes += geometry.commands.length + 128;
    if (bytes > this.maximumBytes || frame.geometry.size > this.maximumCount)
      throw Error('Visible drawing exceeds the renderer cache capacity');
    const next = new Map<bigint, CachedPath>(),
      created: CachedPath[] = [];
    try {
      for (const [id, geometry] of frame.geometry) {
        let entry = this.entries.get(id);
        if (!entry) {
          entry = this.build(geometry);
          created.push(entry);
        }
        next.set(id, entry);
      }
    } catch (error) {
      for (const entry of created) this.remove(entry);
      throw error;
    }
    let finished = false;
    return {
      paths: next,
      commit: () => {
        if (finished) throw Error('Renderer stage already completed');
        finished = true;
        for (const [id, entry] of this.entries) if (!next.has(id)) this.remove(entry);
        this.entries = next;
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
  }
}
