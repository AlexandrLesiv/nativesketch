import { resolve, join } from 'node:path';
export function options(args: string[], root: string) {
  const values = new Map<string, string>();
  for (let i = 0; i < args.length; i += 2) {
    const key = args[i];
    const value = args[i + 1];
    if (
      !['--lane', '--output', '--tools'].includes(key) ||
      !value ||
      value.startsWith('--') ||
      values.has(key)
    )
      throw new Error(
        'Usage: --lane advisories|miri|miri-app|asan --output NEW_DIRECTORY [--tools TOOLCHAIN_ROOT]',
      );
    values.set(key, value);
  }
  const lane = values.get('--lane');
  const output = values.get('--output');
  if (!lane || !['advisories', 'miri', 'miri-app', 'asan'].includes(lane) || !output)
    throw new Error('Explicit lane and fresh output directory are required');
  return {
    lane,
    output: resolve(output),
    tools: resolve(values.get('--tools') ?? join(root, '.build-cache/toolchains')),
  };
}
