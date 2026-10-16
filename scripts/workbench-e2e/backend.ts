import { readdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { controller } from '../lib/context.ts';
import { executeNode } from '../evidence/node.ts';
import { sourceIdentity } from '../evidence/identity.ts';
import { validateDiscovery, validateExecution } from '../evidence/validate.ts';
import { persistExecution } from '../evidence/finalize.ts';
import type { Source, Discovery, Execution } from '../evidence/types.ts';

/** Host integration evidence stays separate from browser and portable unit acceptance. */
export async function backendEvidence(
  root: string,
  output: string,
  source: Source,
  attempt: number,
  signal: AbortSignal = controller.signal,
) {
  signal.throwIfAborted();
  const files = (await readdir(join(root, 'scripts/workbench/tests')))
    .filter((name) => name.endsWith('.test.ts'))
    .sort()
    .map((name) => `scripts/workbench/tests/${name}`);
  const observed = await executeNode(root, files, signal);
  const platform = process.platform === 'darwin' ? 'macos' : process.platform;
  const unchanged = JSON.stringify(source) === JSON.stringify(await sourceIdentity(root));
  const discovery: Discovery = {
    schemaVersion: 1,
    source,
    scopes: [{ runner: 'node', platform, level: 'integration' }],
    tests: observed.tests.map((item) => ({ ...item, platform, level: 'integration' })),
  };
  const execution: Execution = {
    schemaVersion: 1,
    source,
    results: observed.results.map((item) => ({
      ...item,
      attempt,
      platform,
      environment: unchanged
        ? item.environment
        : {
            valid: false,
            reasons: [...item.environment.reasons, 'Source changed during service checks'],
          },
    })),
  };
  signal.throwIfAborted();
  validateDiscovery(discovery);
  validateExecution(execution);
  await writeFile(join(output, 'backend-discovery.json'), JSON.stringify(discovery, null, 2));
  signal.throwIfAborted();
  await persistExecution(execution, signal, join(output, 'backend-results.json'));
  if (
    !execution.results.length ||
    execution.results.some((item) => item.status !== 'passed' || !item.environment.valid)
  )
    throw new Error('Workbench service checks did not pass; see backend-results.json');
  signal.throwIfAborted();
  console.log(`Passed ${execution.results.length} workbench service checks on ${platform}`);
}
