import { resolve, join } from 'node:path';
import { mkdir } from 'node:fs/promises';
import { main, root } from './lib/context.ts';
import { macosJourney } from './e2e/macos.ts';
import { e2eOptions } from './e2e/options.ts';

await main(async () => {
  const options = e2eOptions(process.argv.slice(2));
  const output = options.output
    ? resolve(options.output)
    : join(root, 'artifacts/evidence', `macos-${Date.now()}`);
  await mkdir(resolve(output, '..'), { recursive: true });
  await macosJourney(output, options.execute, options.attempt);
});
