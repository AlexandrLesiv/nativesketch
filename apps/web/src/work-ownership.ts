import type { Engine } from './engine.ts';
import { EngineError } from './transport.ts';
import { words, integers, u64, u32 } from './wire.ts';
interface Entry {
  work: number;
  operation: bigint;
  kind: number;
  encoded: boolean;
  result: number;
  ready: boolean;
  stale: boolean;
}
/** A rejected subsequent call never drops the previously issued work/result. */
export class WorkOwnership {
  private engine: Engine;
  private entries = new Map<number, Entry>();
  constructor(engine: Engine) {
    this.engine = engine;
  }
  issue(work: number) {
    this.entries.set(work, {
      work,
      operation: 0n,
      kind: 0,
      encoded: false,
      result: 0,
      ready: false,
      stale: false,
    });
  }
  info(work: number) {
    const b = this.engine.transport.work(work, 100).bytes;
    const e = this.entries.get(work)!;
    e.operation = u64(b, 72);
    e.kind = u32(b, 96);
    return b;
  }
  call(work: number, op: number, body = new Uint8Array(), input = 0) {
    const r = this.engine.transport.work(work, op, body, input),
      e = this.entries.get(work)!;
    if (r.result) e.result = r.result;
    else if (op === 101) e.encoded = true;
    return r;
  }
  finish(work: number, stale: boolean, effect: (b: Uint8Array) => void) {
    const e = this.entries.get(work)!,
      t = this.engine.transport;
    e.stale ||= stale;
    if (!e.ready) {
      t.complete(e.result);
      e.result = 0;
      e.ready = true;
    }
    let response: Uint8Array | undefined;
    if (e.stale && e.kind !== 4) t.call(33, words(e.operation));
    else if (e.kind !== 4) {
      try {
        response = t.call(32, words(this.engine.session, e.operation));
      } catch (error) {
        if (e.kind !== 2 || !(error instanceof EngineError) || error.status !== 7) throw error;
      }
    }
    this.entries.delete(work);
    if (response) effect(response);
  }
  recover() {
    for (const e of this.entries.values()) {
      if (!e.kind) this.info(e.work);
      if (!e.result && !e.ready) {
        if (e.encoded) this.call(e.work, 104, integers(2, 6));
        else if (e.kind <= 2) this.call(e.work, 105, integers(6, 0));
        else this.call(e.work, e.kind === 3 ? 103 : 106);
      }
      this.finish(e.work, e.kind === 2 || e.kind === 3, () => {});
    }
  }
}
