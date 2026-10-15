import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
import { run } from './process.mjs';
export const root = fileURLToPath(new URL('../../', import.meta.url));
export const controller = new AbortController();
process.once('SIGINT', () => controller.abort());
process.once('SIGTERM', () => controller.abort());
/** @param {string} executable @param {string[]} args @param {Partial<import('./process.mjs').ProcessOptions>} [options] */
export const command = (executable, args, options = {}) =>
  run(executable, args, {
    cwd: root,
    signal: controller.signal,
    ...options,
    env: { CARGO_TARGET_DIR: join(root, 'target'), ...options.env },
  });
/** @param {string} host */
export function requireHost(host) {
  if (process.platform !== host)
    throw new Error(`Requires ${host}; current host is ${process.platform}`);
}
/** @param {() => Promise<unknown>} action */
export async function main(action) {
  try {
    await action();
  } catch (error) {
    console.error(error);
    process.exitCode = 1;
  }
}
