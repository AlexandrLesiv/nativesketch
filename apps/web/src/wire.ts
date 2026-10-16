/** Owned bytes only; opaque identities never pass through Number. */
export const words = (...values: bigint[]) => {
  const b = new Uint8Array(values.length * 8);
  values.forEach((v, i) => view(b).setBigUint64(i * 8, v, true));
  return b;
};
export const integers = (...values: number[]) => {
  const b = new Uint8Array(values.length * 4);
  values.forEach((v, i) => view(b).setUint32(i * 4, v, true));
  return b;
};
export const floats = (...values: number[]) => {
  const b = new Uint8Array(values.length * 8);
  values.forEach((v, i) => view(b).setFloat64(i * 8, v, true));
  return b;
};
export const view = (b: Uint8Array) => new DataView(b.buffer, b.byteOffset, b.byteLength);
export const u64 = (b: Uint8Array, o: number) => view(b).getBigUint64(o, true);
export const u32 = (b: Uint8Array, o: number) => view(b).getUint32(o, true);
export const f64 = (b: Uint8Array, o: number) => view(b).getFloat64(o, true);
export function join(...parts: Uint8Array[]) {
  const b = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let offset = 0;
  for (const p of parts) {
    b.set(p, offset);
    offset += p.length;
  }
  return b;
}
export function packet(op: number, request: bigint, body = new Uint8Array()) {
  const b = new Uint8Array(24);
  b.set([78, 83, 65, 80, 2]);
  view(b).setUint32(8, op, true);
  view(b).setUint32(12, body.length, true);
  view(b).setBigUint64(16, request, true);
  return join(b, body);
}
