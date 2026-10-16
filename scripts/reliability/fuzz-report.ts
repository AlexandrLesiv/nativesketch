import type { Source, Discovery, Execution } from '../evidence/types.ts';
import { validateDiscovery, validateExecution } from '../evidence/validate.ts';
import type { buildFuzzTargets } from './fuzz-targets.ts';
import { json } from './evidence.ts';

export async function fuzzDiscovery(
  output: string,
  source: Source,
  targets: Awaited<ReturnType<typeof buildFuzzTargets>>,
) {
  const discovery: Discovery = {
    schemaVersion: 1,
    source,
    scopes: [{ runner: 'libfuzzer', platform: 'portable', level: 'fuzz' }],
    tests: targets.map((target) => ({
      id: `libfuzzer:sketch-core:${target.name}`,
      runner: 'libfuzzer',
      platform: 'portable',
      level: 'fuzz',
      binarySha256: target.binarySha256,
    })),
  };
  validateDiscovery(discovery);
  await json(output, 'discovery.json', discovery);
}

export async function fuzzExecution(output: string, execution: Execution) {
  validateExecution(execution);
  await json(output, 'results.json', execution);
}
