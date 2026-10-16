import { Transport } from './transport.ts';
import { floats, words, integers, join, u64, u32, f64 } from './wire.ts';
export { floats } from './wire.ts';
export type Action = 'pen' | 'select' | 'previous' | 'next' | 'delete' | 'undo' | 'redo';
export class Engine {
  readonly transport: Transport;
  readonly session: bigint;
  constructor(exports: WebAssembly.Exports) {
    this.transport = new Transport(exports);
    this.session = u64(
      this.transport.call(
        8,
        join(integers(3, 1), floats(36 / 255, 99 / 255, 235 / 255, 1, 4), integers(1, 0)),
      ),
      72,
    );
  }
  isAvailable() {
    return this.transport.available;
  }
  projection() {
    return this.transport.call(3, words(this.session));
  }
  context() {
    const p = this.projection();
    return words(this.session, u64(p, 80), u64(p, 104));
  }
  state() {
    const p = this.projection(),
      flags = u32(p, 156);
    const b = this.transport.call(6, words(this.session));
    return {
      count: u64(p, 112),
      selected: u64(p, 120),
      undo: !!(flags & 2),
      redo: !!(flags & 4),
      dirty: !!(flags & 1),
      interaction: u32(p, 148),
      tool: u32(p, 144) === 1 ? 'pen' : 'select',
      bounds: [72, 80, 88, 96].map((o) => f64(b, o)),
    };
  }
  edit(tag: number, body = new Uint8Array()) {
    return this.transport.call(10, join(this.context(), integers(tag, 0), body));
  }
  action(action: Action) {
    const tag = { pen: 1, select: 1, previous: 2, next: 2, delete: 3, undo: 4, redo: 5 }[action];
    const extra =
      action === 'pen' || action === 'select'
        ? integers(action === 'pen' ? 1 : 2, 0)
        : action === 'previous' || action === 'next'
          ? integers(action === 'previous' ? 1 : 2, 0)
          : new Uint8Array();
    return this.transport.call(11, join(this.context(), integers(tag, 0), extra));
  }
  scale(factor: number) {
    return this.transport.call(11, join(this.context(), integers(6, 0, 1, 0), floats(factor)));
  }
  brush(rgba: number[], width: number) {
    this.edit(2, floats(...rgba, width));
  }
  begin(pointer: bigint, device: number) {
    this.edit(6, join(words(pointer), integers(device, 0)));
  }
  commit(pointer: bigint) {
    this.edit(7, words(pointer));
  }
  cancel() {
    this.edit(11);
  }
  select(x: number, y: number, tolerance: number) {
    this.edit(3, floats(x, y, tolerance));
  }
  beginTransform() {
    this.edit(8);
  }
  transform(scale: number, x: number, y: number) {
    this.edit(9, floats(scale, x, y));
  }
  commitTransform() {
    this.edit(10);
  }
  append(
    pointer: bigint,
    samples: { x: number; y: number; time: number; pressure?: number; id: bigint }[],
  ) {
    const records = samples.map((s) =>
      join(
        integers(0, s.pressure === undefined ? 0 : 1),
        words(s.id),
        floats(s.x, s.y, s.time, s.pressure ?? 0),
      ),
    );
    this.transport.call(
      12,
      join(this.context(), words(pointer), integers(samples.length, 0), ...records),
    );
  }
  file(tag: number) {
    return this.transport.call(20, join(this.context(), integers(tag, 0)));
  }
  reply(effect: bigint, decision: number) {
    return this.transport.call(21, join(this.context(), words(effect), integers(decision, 0)));
  }
  resource(effect: bigint, resource: bigint, read: boolean) {
    return this.transport.call(
      22,
      join(this.context(), words(effect), integers(1, read ? 1 : 2), words(resource, resource, 1n)),
    );
  }
  bounds() {
    return this.transport.call(9, words(this.session));
  }
}
export function resetReplacementChoice(dialog: { returnValue: string }) {
  dialog.returnValue = 'cancel';
}
