import { Buffer } from 'node:buffer';
import { stripVTControlCharacters } from 'node:util';
import { constants } from 'node:fs';
import { realpath, open } from 'node:fs/promises';
import { resolve, relative, isAbsolute } from 'node:path';
import { createHash } from 'node:crypto';
export const hash = (value: string | Uint8Array) =>
  createHash('sha256').update(value).digest('hex');
export function safeText(value: string) {
  return [...stripVTControlCharacters(value)]
    .filter((character) => character.charCodeAt(0) >= 32 || '\t\n\r'.includes(character))
    .join('')
    .replace(/\/(?:Users|home)\/[^\s"'<>),]+/g, '[private-path]')
    .replace(/[A-Za-z]:[\\/](?:Users|home)[\\/][^\s"'<>),]+/gi, '[private-path]')
    .replace(/\b(?:gh[pousr]_|sk-)[A-Za-z0-9_-]{12,}/g, '[redacted-token]');
}
export function sanitize<T>(value: T): T {
  return JSON.parse(
    JSON.stringify(value, (_key, item: unknown) =>
      typeof item === 'string' ? safeText(item) : item,
    ),
  );
}
export async function confined(root: string, id: string) {
  if (!id || isAbsolute(id) || id.includes('\\') || id.split('/').includes('..'))
    throw new Error('Invalid repository identifier');
  const base = await realpath(root);
  const path = await realpath(resolve(base, id));
  const rel = relative(base, path);
  if (!rel || rel.startsWith('..') || isAbsolute(rel)) throw new Error('Path escapes repository');
  return path;
}
export async function boundedRead(root: string, id: string, maximum = 2 * 1024 * 1024) {
  const path = await confined(root, id);
  const file = await open(path, constants.O_RDONLY | constants.O_NOFOLLOW);
  try {
    const info = await file.stat();
    if (!info.isFile() || info.size > maximum)
      throw new Error('File missing or exceeds byte bound');
    const buffer = Buffer.alloc(maximum + 1);
    let length = 0;
    while (length < buffer.length) {
      const read = await file.read(buffer, length, buffer.length - length, null);
      if (!read.bytesRead) break;
      length += read.bytesRead;
    }
    if (length > maximum) throw new Error('File exceeds byte bound');
    return buffer.subarray(0, length);
  } finally {
    await file.close();
  }
}
