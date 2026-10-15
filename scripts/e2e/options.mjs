import { parseArgs } from 'node:util';

/** Foreground input is opt-in; parsing options never builds or launches anything.
 * @param {string[]} args */
export function e2eOptions(args) {
  const { values } = parseArgs({
    args,
    options: {
      platform: { type: 'string' },
      output: { type: 'string' },
      attempt: { type: 'string', default: '1' },
      interactive: { type: 'boolean' },
      'prepare-only': { type: 'boolean' },
    },
  });
  const attempt = Number(values.attempt);
  if (values.platform !== 'macos' || !Number.isSafeInteger(attempt) || attempt < 1)
    throw new Error(
      'Usage: node scripts/test-e2e.mjs --platform macos [--output NEW_DIRECTORY] [--attempt N] [--interactive | --prepare-only]',
    );
  if (values.interactive && values['prepare-only'])
    throw new Error('--interactive and --prepare-only are mutually exclusive');
  return { output: values.output, attempt, execute: values.interactive === true };
}
