import { resolve } from 'node:path';
import { isContained } from '../lib/paths.ts';
export function browserOptions(args: string[], root: string) {
  let output = '',
    attempt = 1;
  while (args.length) {
    const flag = args.shift(),
      value = args.shift();
    if (!value || !['--output', '--attempt'].includes(flag ?? ''))
      throw new Error('Use --output artifacts/... [--attempt N]; headed mode is not supported');
    if (flag === '--output') output = resolve(root, value);
    else {
      if (!/^[1-9]\d*$/.test(value)) throw new Error('Invalid attempt');
      attempt = Number(value);
    }
  }
  if (
    !output ||
    !isContained(resolve(root, 'artifacts'), output) ||
    output === resolve(root, 'artifacts')
  )
    throw new Error('Choose a new output directory below artifacts');
  if (!Number.isSafeInteger(attempt)) throw new Error('Invalid attempt');
  return { output, attempt };
}
