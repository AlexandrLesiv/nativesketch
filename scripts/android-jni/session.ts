import { persistJson } from '../evidence/finalize.ts';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
export async function exclusiveDirectory(path: string): Promise<void> {
  await mkdir(dirname(path), { recursive: true });
  await mkdir(path);
}
export function assertStable(initial: unknown, after: unknown): void {
  if (JSON.stringify(initial) !== JSON.stringify(after))
    throw new Error('Source or library changed during execution');
}
export async function captureStage(
  output: string,
  name: string,
  work: () => Promise<string>,
): Promise<string> {
  try {
    const log = await work();
    await writeFile(join(output, `${name}.log`), log);
    return log;
  } catch (error) {
    await writeFile(join(output, `${name}.log`), String(error));
    throw error;
  }
}

/** Metadata first; the final record is the only authoritative qualification. */
export async function publishQualification(
  output: string,
  signal: AbortSignal,
  issues: string[],
  records: () => Record<string, unknown>,
): Promise<void> {
  const entries = Object.entries(records());
  const final = entries.at(-1);
  if (!final) throw new Error('Missing qualification record');
  try {
    for (const [name, value] of entries) await persistJson(value, signal, join(output, name));
  } catch (error) {
    issues.push(signal.aborted ? 'Qualification cancelled before publication' : String(error));
    // Raw metadata remains intact. A failure record never occupies the success filename.
    await persistJson(
      records()[final[0]],
      new AbortController().signal,
      join(output, `failed-${final[0]}`),
    );
    if (!signal.aborted) throw error;
  }
}
