import { resolve } from 'node:path';
export function workbenchOptions(args: string[], root: string) {
  const [mode, ...flags] = args;
  if (!['serve', 'snapshot', 'export'].includes(mode ?? ''))
    throw new Error('Usage: workbench.ts serve [--port PORT] | snapshot|export --output NEW_FILE');
  const values = new Map<string, string>();
  for (let index = 0; index < flags.length; index += 2) {
    const key = flags[index];
    const value = flags[index + 1];
    if (
      !['--port', '--output'].includes(key) ||
      !value ||
      value.startsWith('--') ||
      values.has(key)
    )
      throw new Error('Expected unique --port or --output values');
    values.set(key, value);
  }
  if (mode === 'serve') {
    if (values.has('--output')) throw new Error('serve does not write an export');
    const raw = values.get('--port') ?? '0';
    const port = Number(raw);
    if (!/^\d+$/.test(raw) || !Number.isInteger(port) || port < 0 || port > 65535)
      throw new Error('Port must be an integer from 0 to 65535');
    return { mode: 'serve' as const, port };
  }
  const output = values.get('--output');
  if (!output || values.has('--port'))
    throw new Error('snapshot/export require only --output NEW_FILE');
  return { mode: mode as 'snapshot' | 'export', output: resolve(root, output) };
}
