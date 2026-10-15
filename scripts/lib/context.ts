import type { ProcessOptions } from './process.ts';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
import { run } from './process.ts';
export const root = fileURLToPath(new URL('../../', import.meta.url));
export const controller = new AbortController();
process.once('SIGINT', () => controller.abort());
process.once('SIGTERM', () => controller.abort());

export const command = (
  executable: string,
  args: string[],
  options: Partial<ProcessOptions> = {},
) =>
  run(executable, args, {
    cwd: root,
    signal: controller.signal,
    ...options,
    env: { CARGO_TARGET_DIR: join(root, 'target'), ...options.env },
  });

export function requireHost(host: string) {
  if (process.platform !== host)
    throw new Error(`Requires ${host}; current host is ${process.platform}`);
}

export async function main(action: () => Promise<unknown>) {
  try {
    await action();
  } catch (error) {
    console.error(error);
    process.exitCode = 1;
  }
}
