import type { Engine } from './engine.ts';
import { words, integers, floats, join, u64, u32, f64 } from './wire.ts';
export interface Stroke {
  id: bigint;
  order: bigint;
  geometry: bigint;
  scale: number;
  tx: number;
  ty: number;
}
export interface Geometry {
  origin: number[];
  commands: Uint8Array;
}
export interface Frame {
  strokes: Stroke[];
  geometry: Map<bigint, Geometry>;
}
const descriptor = (b: Uint8Array, o: number): Stroke => ({
  id: u64(b, o),
  order: u64(b, o + 8),
  geometry: u64(b, o + 16),
  scale: f64(b, o + 24),
  tx: f64(b, o + 32),
  ty: f64(b, o + 40),
});
/** One published lease plus one temporary staging lease. Copies are bounded to 32 MiB. */
export class Scene {
  private engine: Engine;
  private maximumBytes: number;
  private lease = 0n;
  private root = 0n;
  private document = 0n;
  private committed: Stroke[] = [];
  frame: Frame = { strokes: [], geometry: new Map() };
  constructor(engine: Engine, maximumBytes = 32 * 1024 * 1024) {
    this.engine = engine;
    this.maximumBytes = maximumBytes;
  }
  update(viewport: number[]) {
    const t = this.engine.transport;
    const info = t.call(40, join(words(this.engine.session, this.lease), floats(...viewport)));
    const next = u64(info, 72);
    if (u32(info, 120)) return this.frame;
    try {
      const count = Number(u64(info, 112)),
        root = u64(info, 104),
        document = u64(info, 80);
      const committed: Stroke[] =
        root === this.root && document === this.document ? this.committed : [];
      if (root === this.root && document === this.document && committed.length !== count)
        throw Error('Inconsistent scene root');
      for (let first = committed.length; first < count;) {
        const page = t.call(41, join(words(next), integers(first, 1365))),
          n = u32(page, 72);
        if (!n) throw Error('Incomplete scene descriptors');
        for (let i = 0; i < n; ++i) committed.push(descriptor(page, 80 + i * 48));
        first += n;
      }
      const strokes = [...committed];
      if (u32(info, 128)) {
        const hidden = u64(info, 176);
        const index = strokes.findIndex((s) => s.id === hidden);
        if (index >= 0) strokes.splice(index, 1);
        if (u32(info, 132)) strokes.push(descriptor(info, 240));
      }
      if (u32(info, 124)) strokes.push(descriptor(info, 192));
      strokes.sort((a, b) => (a.order < b.order ? -1 : a.order > b.order ? 1 : 0));
      const geometry = new Map<bigint, Geometry>();
      let bytes = 0;
      for (const s of strokes) {
        if (geometry.has(s.geometry)) continue;
        let g = this.frame.geometry.get(s.geometry);
        if (!g) {
          const info = t.call(42, words(next, s.geometry));
          const count = Number(u64(info, 120));
          if (bytes + count * 88 > this.maximumBytes)
            throw Error('Scene exceeds browser geometry capacity');
          const commands = new Uint8Array(count * 88);
          for (let first = 0; first < count;) {
            const page = t.call(43, join(words(next, s.geometry), integers(first, 744)));
            const n = u32(page, 72);
            if (!n) throw Error('Incomplete scene geometry');
            commands.set(page.subarray(80), first * 88);
            first += n;
          }
          g = { origin: [f64(info, 72), f64(info, 80)], commands };
        }
        bytes += g.commands.length;
        if (bytes > this.maximumBytes) throw Error('Scene exceeds browser geometry capacity');
        geometry.set(s.geometry, g);
      }
      const previous = this.lease;
      if (previous) t.call(44, words(previous));
      this.frame = { strokes, geometry };
      this.lease = next;
      this.root = root;
      this.document = document;
      this.committed = committed;
      return this.frame;
    } catch (error) {
      t.call(44, words(next));
      throw error;
    }
  }
  dispose() {
    if (this.lease) this.engine.transport.call(44, words(this.lease));
    this.lease = 0n;
    this.root = 0n;
    this.document = 0n;
    this.committed = [];
    this.frame = { strokes: [], geometry: new Map() };
  }
}
