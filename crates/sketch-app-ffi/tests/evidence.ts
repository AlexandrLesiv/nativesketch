import { mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { randomUUID } from 'node:crypto';
import { arch, release } from 'node:os';
import { command, root } from '../../../scripts/lib/context.ts';
import { fileHash, sourceIdentity } from '../../../scripts/evidence/identity.ts';
import { validateDiscovery, validateExecution } from '../../../scripts/evidence/validate.ts';
import type { Discovery, Execution } from '../../../scripts/evidence/types.ts';

export async function evidence() {
  const source = await sourceIdentity(root);
  const directory = join(
    root,
    'artifacts/evidence/portable-exports',
    Date.now() + '-' + randomUUID(),
  );
  await mkdir(directory, { recursive: true });
  const discovery: Discovery = { schemaVersion: 1, source, tests: [] };
  const execution: Execution = { schemaVersion: 1, source, results: [] };
  const toolchain = (await command('rustc', ['--version'], { capture: true })).trim();
  async function record(
    id: string,
    platform: string,
    binary: string,
    action: () => Promise<string>,
  ) {
    const binarySha256 = await fileHash(binary);
    const runner = id.split(':')[0];
    discovery.tests.push({ id, runner, platform, level: 'bridge', binarySha256 });
    const start = performance.now(),
      timestamp = new Date().toISOString();
    let failure: unknown,
      output = '';
    try {
      output = await action();
    } catch (error) {
      failure = error;
      output = String(error);
    }
    const valid = JSON.stringify(await sourceIdentity(root)) === JSON.stringify(source);
    const artifact = runner + '.log';
    await writeFile(join(directory, artifact), output);
    execution.results.push({
      testId: id,
      attempt: 1,
      status: failure ? 'failed' : 'passed',
      durationMs: performance.now() - start,
      timestamp,
      binarySha256,
      platform,
      toolchain: toolchain + '; Node ' + process.versions.node,
      device: process.platform + '/' + arch(),
      os: release(),
      inputRoute: 'portable-export-conformance',
      environment: { valid, reasons: valid ? [] : ['Source changed during conformance run'] },
      artifacts: [artifact],
      output,
    });
    validateDiscovery(discovery);
    validateExecution(execution);
    await writeFile(join(directory, 'discovery.json'), JSON.stringify(discovery, null, 2));
    await writeFile(join(directory, 'execution.json'), JSON.stringify(execution, null, 2));
    console.log(output.trim());
    console.log('Export evidence: ' + directory);
    if (failure) throw failure;
    if (!valid) throw new Error('Source changed during conformance run');
  }
  return { directory, record };
}
