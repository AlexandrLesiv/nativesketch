import { packet, words, u32 } from './wire.ts';
export class EngineError extends Error {
  readonly status: number;
  constructor(status: number) {
    super(`Drawing operation failed (${status}).`);
    this.status = status;
  }
}
export class Transport {
  readonly memory: WebAssembly.Memory;
  private exports: WebAssembly.Exports;
  private request = 0n;
  app = 0;
  available = true;
  private interrupted: ReturnType<Transport['invoke']> | undefined;
  constructor(exports: WebAssembly.Exports) {
    this.exports = exports;
    if (!(exports.memory instanceof WebAssembly.Memory)) throw Error('Missing engine memory');
    this.memory = exports.memory;
    const limits = words(1n, 8n, 16n, 32n, 134217728n, 67108864n, 8n, 134217728n);
    this.app = this.using(this.message(0, limits), (p) => this.invoke('app_new', [p]).primary);
  }
  message(op: number, body = new Uint8Array()) {
    return packet(op, ++this.request, body);
  }
  invoke(name: string, args: number[] = []) {
    if (!this.available) throw Error('Drawing engine is unavailable; reload the page.');
    try {
      const fn = this.exports['ns_v2_wasm_' + name];
      if (typeof fn !== 'function') throw Error('Missing engine operation');
      const b = new Uint8Array(128);
      const status: unknown = fn(...args);
      const resultPtr = this.exports.ns_v2_wasm_result_ptr;
      if (typeof resultPtr !== 'function') throw Error('Missing engine result');
      const ptr: unknown = resultPtr();
      if (typeof ptr !== 'number') throw Error('Invalid engine result');
      b.set(new Uint8Array(this.memory.buffer, ptr, 128));
      if (status !== 0 || u32(b, 0))
        throw new EngineError(typeof status === 'number' ? status : u32(b, 0));
      return {
        flags: u32(b, 4),
        primary: u32(b, 8),
        reply: u32(b, 12),
        work: u32(b, 16),
        result: u32(b, 20),
        count: u32(b, 24),
      };
    } catch (error) {
      if (error instanceof WebAssembly.RuntimeError) this.available = false;
      throw error;
    }
  }
  using<T>(bytes: Uint8Array, fn: (id: number) => T): T {
    const id = this.invoke('upload_new', [bytes.length]).primary;
    try {
      const v = this.invoke('upload_view', [id]);
      new Uint8Array(this.memory.buffer, v.primary, v.count).set(bytes);
      return fn(id);
    } finally {
      if (this.available) this.invoke('upload_free', [id]);
    }
  }
  /** Reserve the maximum v2 response before mutation/transfer. Copying cannot allocate Wasm storage. */
  private exchange(name: string, args: number[], bytes: Uint8Array, input?: number) {
    const owned = new Uint8Array(65616);
    return this.using(bytes, (p) =>
      this.using(new Uint8Array(owned.length), (scratch) => {
        const result = this.invoke(name, [...args, p, ...(input === undefined ? [] : [input])]);
        this.interrupted = result;
        try {
          const n = this.invoke('buffer_len', [result.reply]).count;
          if (n > owned.length) throw Error('Portable response exceeds its protocol limit');
          this.invoke('buffer_copy', [result.reply, 0, scratch, n]);
          const v = this.invoke('upload_view', [scratch]);
          owned.set(new Uint8Array(this.memory.buffer, v.primary, n));
          const response = { ...result, bytes: owned.subarray(0, n) };
          this.interrupted = undefined;
          return response;
        } catch (error) {
          this.available = false;
          throw error;
        } finally {
          if (this.available) this.invoke('buffer_free', [result.reply]);
        }
      }),
    );
  }
  call(op: number, body = new Uint8Array()) {
    return this.exchange('app_call', [this.app], this.message(op, body)).bytes;
  }
  work(handle: number, op: number, body = new Uint8Array(), input = 0) {
    return this.exchange('work_call', [handle], this.message(op, body), input);
  }
  complete(result: number) {
    this.using(this.message(31), (p) => {
      const receipt = this.invoke('app_complete', [this.app, p, result]);
      this.invoke('buffer_free', [receipt.reply]);
    });
  }
  take() {
    return this.using(this.message(30), (p) => {
      const receipt = this.invoke('app_take_work', [this.app, p]);
      this.invoke('buffer_free', [receipt.reply]);
      return receipt.work;
    });
  }
}
