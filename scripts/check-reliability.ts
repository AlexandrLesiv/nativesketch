import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { release } from 'node:os';
import { root, main } from './lib/context.ts';
import { sourceIdentity } from './evidence/identity.ts';
import { freshOutput, json } from './reliability/evidence.ts';
import { options } from './reliability/options.ts';
import { executeLane } from './reliability/lanes.ts';
import type { Pins } from './reliability/lanes.ts';
import { requireUnchanged } from './reliability/validation.ts';

await main(async () => {
  const args = options(process.argv.slice(2), root);
  const pins: Pins = JSON.parse(
    await readFile(join(root, 'quality/reliability-tools.json'), 'utf8'),
  );
  await freshOutput(args.output);
  const startedAt = new Date().toISOString();
  const source = await sourceIdentity(root);
  await json(args.output, 'identity.json', {
    source,
    startedAt,
    lane: args.lane,
    node: process.version,
    platform: process.platform,
    architecture: process.arch,
    os: release(),
  });
  try {
    if (process.versions.node !== pins.node) throw new Error('Pinned Node version required');
    const evidence = await executeLane(args.lane, root, args.output, args.tools, pins);
    requireUnchanged(source, await sourceIdentity(root));
    await json(args.output, 'result.json', {
      status: 'passed',
      source,
      lane: args.lane,
      startedAt,
      finishedAt: new Date().toISOString(),
      evidence,
    });
  } catch (error) {
    await json(args.output, 'result.json', {
      status: 'failed',
      source,
      lane: args.lane,
      startedAt,
      finishedAt: new Date().toISOString(),
      reason: String(error),
    });
    throw error;
  }
});
