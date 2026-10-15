import { resolve, join } from 'node:path';
import { mkdir } from 'node:fs/promises';
import { main, root } from './lib/context.mjs';
import { macosJourney } from './e2e/macos.mjs';
import { e2eOptions } from './e2e/options.mjs';

await main(async () => {
  const options = e2eOptions(process.argv.slice(2));
  const output = options.output
    ? resolve(options.output)
    : join(root, 'artifacts/evidence', `macos-${Date.now()}`);
  await mkdir(resolve(output, '..'), { recursive: true });
  await macosJourney(output, options.execute, options.attempt);
});
