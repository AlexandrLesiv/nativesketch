import { readFileSync } from 'node:fs';
export type Schema = {
  type?: string;
  properties?: Record<string, Schema>;
  required?: string[];
  items?: Schema;
  minItems?: number;
  uniqueItems?: boolean;
  minLength?: number;
  minimum?: number;
  pattern?: string;
  enum?: unknown[];
  const?: unknown;
  anyOf?: Schema[];
  format?: string;
};
/** Dependency-free validator for the explicit keyword subset used by the repository schemas. */
export function validateSchema(value: unknown, schema: Schema, path: string = '$') {
  const fail = (reason: string) => {
    throw new Error(`${path}: ${reason}`);
  };
  if (schema.anyOf) {
    if (
      !schema.anyOf.some((candidate) => {
        try {
          validateSchema(value, candidate, path);
          return true;
        } catch {
          return false;
        }
      })
    )
      fail('no schema alternative matched');
    return;
  }
  if ('const' in schema && value !== schema.const) fail('unexpected constant');
  if (schema.enum && !schema.enum.includes(value)) fail('unknown value');
  switch (schema.type) {
    case 'null':
      if (value !== null) fail('expected null');
      break;
    case 'boolean':
      if (typeof value !== 'boolean') fail('expected boolean');
      break;
    case 'integer':
    case 'number':
      if (typeof value !== 'number' || !Number.isFinite(value)) fail('expected finite number');
      if (schema.type === 'integer' && !Number.isInteger(value)) fail('expected integer');
      if (schema.minimum !== undefined && (value as number) < schema.minimum) fail('below minimum');
      break;
    case 'string':
      if (typeof value !== 'string') fail('expected string');
      if (schema.minLength && String(value).length < schema.minLength) fail('empty string');
      if (schema.pattern && !new RegExp(schema.pattern).test(String(value)))
        fail('pattern mismatch');
      if (
        schema.format === 'date-time' &&
        (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$/.test(String(value)) ||
          !Number.isFinite(Date.parse(String(value))))
      )
        fail('invalid UTC timestamp');
      break;
    case 'array': {
      if (!Array.isArray(value)) fail('expected array');
      const values = value as unknown[];
      if (schema.minItems && values.length < schema.minItems) fail('too few items');
      if (
        schema.uniqueItems &&
        new Set(values.map((v) => JSON.stringify(v))).size !== values.length
      )
        fail('duplicate items');
      values.forEach((v, i) => {
        if (schema.items) validateSchema(v, schema.items, `${path}[${i}]`);
      });
      break;
    }
    case 'object': {
      if (!value || typeof value !== 'object' || Array.isArray(value)) fail('expected object');
      const record = value as Record<string, unknown>;
      for (const key of schema.required ?? []) if (!(key in record)) fail(`missing ${key}`);
      for (const [key, child] of Object.entries(schema.properties ?? {}))
        if (key in record) validateSchema(record[key], child, `${path}.${key}`);
      break;
    }
  }
}

export function loadSchema(name: string) {
  return JSON.parse(
    readFileSync(new URL(`../../quality/schema/${name}.schema.json`, import.meta.url), 'utf8'),
  ) as Schema;
}
