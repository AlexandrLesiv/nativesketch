import { mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { run } from '../lib/process.ts';
import { ProcessFailure } from '../lib/process-failure.ts';
import type { ProcessOptions } from '../lib/process.ts';

export async function freshOutput(output: string) {
  await mkdir(output, { recursive: false });
}
export async function json(output: string, name: string, value: unknown) {
  await writeFile(join(output, name), JSON.stringify(value, null, 2) + '\n');
}
export async function recorded(
  output: string,
  name: string,
  executable: string,
  args: string[],
  options: ProcessOptions,
) {
  await json(output, `${name}.command.json`, {
    executable,
    args,
    cwd: options.cwd,
    environment: options.env,
    timeout: options.timeout ?? 600_000,
  });
  try {
    const text = await run(executable, args, { ...options, capture: true });
    await writeFile(join(output, `${name}.log`), text + '\n');
    return text;
  } catch (error) {
    await writeFile(
      join(output, `${name}.log`),
      error instanceof ProcessFailure ? error.output : String(error),
    );
    await json(output, `${name}.failure.json`, {
      message: String(error),
      ...(error instanceof ProcessFailure
        ? { exitCode: error.exitCode, signal: error.signal, reason: error.reason }
        : {}),
    });
    throw error;
  }
}
