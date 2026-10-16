import { WorkOwnership } from './work-ownership.ts';
import type { Engine } from './engine.ts';
import { words, integers, join, u64, u32 } from './wire.ts';
export interface Source {
  size: number;
  arrayBuffer(): Promise<ArrayBuffer>;
}
export class FileWork {
  private engine: Engine;
  private ownership: WorkOwnership;
  private resources = new Map<bigint, Source | undefined>();
  private next = 0n;
  private pending = new Set<Promise<void>>();
  stopping = false;
  constructor(engine: Engine) {
    this.engine = engine;
    this.ownership = new WorkOwnership(engine);
  }
  supply(effect: bigint, source?: Source) {
    if (this.resources.size >= 8) throw Error('Too many file operations are pending');
    const id = ++this.next;
    const response = this.engine.resource(effect, id, !!source);
    this.resources.set(id, source);
    return response;
  }
  releases() {
    for (;;) {
      const b = this.engine.transport.call(34);
      if (!u32(b, 72)) break;
      if (!this.resources.delete(u64(b, 96))) throw Error('Unknown file resource release');
    }
  }
  drain(
    onEffect: (b: Uint8Array) => void,
    dispatch: (bytes: Uint8Array) => void,
    failure: (e: unknown) => void,
  ) {
    const t = this.engine.transport;
    const started: Promise<void>[] = [];
    for (let work = t.take(); work; work = t.take()) {
      this.ownership.issue(work);
      const task = this.execute(work, onEffect, dispatch).catch(failure);
      this.pending.add(task);
      started.push(task);
      void task.finally(() => this.pending.delete(task));
    }
    this.releases();
    return Promise.all(started);
  }
  private async execute(
    work: number,
    effect: (b: Uint8Array) => void,
    dispatch: (b: Uint8Array) => void,
  ) {
    const t = this.engine.transport,
      info = this.ownership.info(work);
    const kind = u32(info, 96);
    let result: ReturnType<typeof t.work>;
    let stale = false;
    if (kind === 1) {
      result = this.ownership.call(work, 101);
      if (!result.result) {
        const size = Number(u64(result.bytes, 80)),
          bytes = new Uint8Array(size);
        for (let offset = 0; offset < size;) {
          const page = t.work(work, 107, join(words(BigInt(offset)), integers(65536, 0))).bytes;
          const n = u32(page, 72);
          if (!n) throw Error('Incomplete encoded file');
          bytes.set(page.subarray(80), offset);
          offset += n;
        }
        let outcome = integers(4, 0);
        try {
          dispatch(bytes);
        } catch {
          outcome = integers(2, 4);
        }
        result = this.ownership.call(work, 104, outcome);
      }
    } else if (kind === 2) {
      const source = this.resources.get(u64(info, 112));
      const before = this.engine.projection();
      try {
        if (!source || source.size > Number(u64(info, 136))) throw Error('File exceeds read limit');
        const bytes = new Uint8Array(await source.arrayBuffer());
        const input = t.using(bytes, (p) => t.invoke('work_copy_input', [work, p]).primary);
        try {
          result = this.ownership.call(work, 102, new Uint8Array(), input);
        } catch (error) {
          t.invoke('input_free', [input]);
          throw error;
        }
      } catch {
        result = this.ownership.call(work, 105, integers(2, 0));
      }
      const after = this.engine.projection();
      stale =
        this.stopping ||
        u64(before, 80) !== u64(after, 80) ||
        u64(before, 88) !== u64(after, 88) ||
        u64(after, 104) !== 0n;
    } else result = this.ownership.call(work, kind === 3 ? 103 : 106);
    this.ownership.finish(work, stale, effect);
    this.releases();
    for (let retired = t.take(); retired; retired = t.take()) {
      this.ownership.issue(retired);
      const info = this.ownership.info(retired);
      if (u32(info, 96) !== 4) throw Error('Unexpected concurrent work');
      this.ownership.call(retired, 106);
      this.ownership.finish(retired, false, effect);
    }
    this.releases();
  }
  async shutdown() {
    this.stopping = true;
    await Promise.all(this.pending);
    this.engine.cancel();
    this.ownership.recover();
    const r = this.engine.file(6);
    const response = this.engine.reply(u64(r, 240), u32(r, 232) === 3 ? 3 : 5);
    if (u32(response, 232) === 6) this.engine.reply(u64(response, 240), 5);
    this.engine.transport.call(2, words(this.engine.session));
    for (let work = this.engine.transport.take(); work; work = this.engine.transport.take())
      this.engine.transport.complete(this.engine.transport.work(work, 106).result);
    this.releases();
    this.engine.transport.invoke('app_free', [this.engine.transport.app]);
    this.engine.transport.available = false;
  }
}
