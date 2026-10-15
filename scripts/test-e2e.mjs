import { resolve, join } from 'node:path';
import { mkdir } from 'node:fs/promises';
import { parseArgs } from 'node:util';
import { main, root } from './lib/context.mjs';
import { macosJourney } from './e2e/macos.mjs';

await main(async () => {
  const { values } = parseArgs({
    options: {
      platform: { type: 'string' },
      output: { type: 'string' },
      attempt: { type: 'string', default: '1' },
      'prepare-only': { type: 'boolean' },
    },
  });
  const attempt = Number(values.attempt);
  if (values.platform !== 'macos' || !Number.isSafeInteger(attempt) || attempt < 1)
    throw new Error(
      'Usage: node scripts/test-e2e.mjs --platform macos [--output NEW_DIRECTORY] [--attempt N] [--prepare-only]',
    );
  const output = values.output
    ? resolve(values.output)
    : join(root, 'artifacts/evidence', `macos-${Date.now()}`);
  await mkdir(resolve(output, '..'), { recursive: true });
  await macosJourney(output, !values['prepare-only'], attempt);
});
