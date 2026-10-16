import type { Execution } from './types.ts';
import { writeFile } from 'node:fs/promises';
import { linkSync, rmSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';
import { randomUUID } from 'node:crypto';

/** A complete staged file becomes evidence at one synchronous, exclusive publication point. */
export async function persistExecution(
  execution: Execution,
  signal: AbortSignal,
  destination: string,
  stage: (path: string, contents: string) => Promise<void> = (path, contents) =>
    writeFile(path, contents, { flag: 'wx', mode: 0o600 }),
) {
  return persistJson(execution, signal, destination, stage);
}

/** One staged record becomes evidence at one cancellation-checked publication point. */
export async function persistJson(
  value: unknown,
  signal: AbortSignal,
  destination: string,
  stage: (path: string, contents: string) => Promise<void> = (path, contents) =>
    writeFile(path, contents, { flag: 'wx', mode: 0o600 }),
) {
  signal.throwIfAborted();
  const pending = join(dirname(destination), `.${basename(destination)}.${randomUUID()}.pending`);
  try {
    await stage(pending, JSON.stringify(value, null, 2));
    signal.throwIfAborted();
    linkSync(pending, destination);
  } finally {
    rmSync(pending, { force: true });
  }
}
