import type { IncomingMessage } from 'node:http';
import { timingSafeEqual } from 'node:crypto';
export function authorize(
  request: IncomingMessage,
  authority: string,
  capability: string,
  bootstrap = false,
) {
  if (request.headers.host !== authority) throw new Error('Host rejected');
  const origin = `http://${authority}`;
  if (request.headers.origin && request.headers.origin !== origin)
    throw new Error('Origin rejected');
  if (request.headers['sec-fetch-site'] && request.headers['sec-fetch-site'] !== 'same-origin')
    throw new Error('Fetch site rejected');
  if (bootstrap && request.headers['sec-fetch-site'] !== 'same-origin')
    throw new Error('Session requires same-origin browser fetch');
  if (request.method !== 'GET' && request.headers.origin !== origin)
    throw new Error('Mutation requires same-origin Origin');
  if (!bootstrap) {
    const header = request.headers['x-workbench-capability'];
    if (
      typeof header !== 'string' ||
      Buffer.byteLength(header) !== Buffer.byteLength(capability) ||
      !timingSafeEqual(Buffer.from(header), Buffer.from(capability))
    )
      throw new Error('Capability rejected');
  }
}
export async function body(request: IncomingMessage, keys: string[]) {
  if (request.headers['content-type'] !== 'application/json')
    throw new Error('Expected application/json');
  let bytes = 0;
  const chunks: Buffer[] = [];
  for await (const chunk of request) {
    const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    bytes += buffer.length;
    if (bytes > 4096) throw new Error('Request body exceeds bound');
    chunks.push(buffer);
  }
  const value: unknown = JSON.parse(Buffer.concat(chunks).toString());
  if (!value || typeof value !== 'object' || Array.isArray(value))
    throw new Error('Expected object body');
  const object = value as Record<string, unknown>;
  if (
    Object.keys(object).length !== keys.length ||
    keys.some((key) => typeof object[key] !== 'string')
  )
    throw new Error('Unexpected request fields');
  return object;
}
