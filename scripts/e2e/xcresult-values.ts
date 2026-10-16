export function record(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value))
    throw new Error('Expected xcresult object');
  return value as Record<string, unknown>;
}
export function wrapped(value: unknown, type: string): Record<string, unknown> {
  const item = record(value);
  if (record(item._type)._name !== type) throw new Error(`Expected xcresult ${type}`);
  return item;
}
export function list(value: unknown): unknown[] {
  const item = wrapped(value, 'Array');
  if (item._values === undefined) return [];
  if (!Array.isArray(item._values)) throw new Error('Invalid xcresult array');
  return item._values;
}
export function string(value: unknown, type = 'String'): string {
  const text = wrapped(value, type)._value;
  if (typeof text !== 'string') throw new Error('Invalid xcresult string');
  return text;
}
export function integer(value: unknown): number {
  const text = string(value, 'Int');
  if (!/^(0|[1-9][0-9]*)$/.test(text)) throw new Error('Invalid xcresult integer');
  return count(Number(text));
}
export function count(value: unknown): number {
  if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < 0)
    throw new Error('Invalid test count');
  return value;
}
