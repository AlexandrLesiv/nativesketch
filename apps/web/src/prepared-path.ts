import type { Engine } from './engine.ts';
import type { Geometry, Stroke } from './scene.ts';
import { words, integers, join, u64, u32 } from './wire.ts';
/** Retain a rejected release; never issue a second preparation while its lease is held. */
export class PreparedPath {
  private engine: Engine;
  private held = 0n;
  constructor(engine: Engine) {
    this.engine = engine;
  }
  release() {
    if (!this.held) return;
    this.engine.transport.call(47, words(this.held));
    this.held = 0n;
  }
  read(
    scene: bigint,
    stroke: Stroke,
    width: number,
    height: number,
    budget: number,
  ): Geometry | null {
    this.release();
    const reply = this.engine.transport.call(
      45,
      join(words(scene), integers(stroke.selector, stroke.index, width, height)),
    );
    this.held = u64(reply, 80);
    try {
      const mode = u32(reply, 72),
        count = u64(reply, 120);
      if (
        u64(reply, 88) !== scene ||
        u64(reply, 96) !== stroke.geometry ||
        u64(reply, 104) !== stroke.id ||
        u64(reply, 112) !== stroke.order ||
        u32(reply, 128) !== width ||
        u32(reply, 132) !== height ||
        u32(reply, 76)
      )
        throw Error('Inconsistent prepared path provenance');
      if (!mode && !this.held && !count) return null;
      if (mode !== 1 || !this.held || count > BigInt(Math.floor(budget / 88)))
        throw Error('Prepared path exceeds renderer capacity');
      const commands = new Uint8Array(Number(count) * 88);
      for (let first = 0; first < Number(count);) {
        const page = this.engine.transport.call(46, join(words(this.held), integers(first, 744)));
        const n = u32(page, 72);
        if (!n || n > 744 || first + n > Number(count) || page.length !== 80 + n * 88)
          throw Error('Incomplete prepared path page');
        commands.set(page.subarray(80), first * 88);
        first += n;
      }
      return { origin: [0, 0], commands };
    } finally {
      this.release();
    }
  }
}
