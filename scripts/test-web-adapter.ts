import { readdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { platform, release, arch } from 'node:os';
import { root, controller, command, main } from './lib/context.ts';
import { browserOptions } from './browser-e2e/options.ts';
import { prepareOutput } from './browser-e2e/output.ts';
import { adapterIdentity } from './web-evidence/identity.ts';
import { sourceIdentity } from './evidence/identity.ts';
import { executeNode } from './evidence/node.ts';
import { persistExecution } from './evidence/finalize.ts';
import { validateDiscovery, validateExecution } from './evidence/validate.ts';
import type { Discovery, Execution } from './evidence/types.ts';

await main(async () => {
  const { output, attempt } = browserOptions(process.argv.slice(2), root);
  await prepareOutput(root, output);
  const save = (name: string, value: unknown) =>
    writeFile(join(output, name), JSON.stringify(value, null, 2));
  try {
    const before = await sourceIdentity(root);
    try {
      const log = await command(process.execPath, ['scripts/build.ts', '--platform', 'web'], {
        capture: true,
      });
      await writeFile(join(output, 'build.log'), log);
    } catch (error) {
      await writeFile(join(output, 'build.log'), String(error));
      throw error;
    }
    const identity = await adapterIdentity(root);
    if (JSON.stringify(before) !== JSON.stringify(identity.source))
      throw new Error('Source changed during production Wasm build');
    await save('identity.json', { ...identity, node: process.version, attempt });
    const files = (await readdir(join(root, 'apps/web/tests')))
      .filter((name) => name.endsWith('.test.ts'))
      .sort()
      .map((name) => `apps/web/tests/${name}`);
    if (!files.length) throw new Error('No web adapter test files discovered');
    const observed = await executeNode(root, files, controller.signal);
    await save('node-observations.json', observed);
    const unchanged = JSON.stringify(identity) === JSON.stringify(await adapterIdentity(root));
    const discovery: Discovery = {
      schemaVersion: 1,
      source: identity.source,
      scopes: [{ runner: 'node', platform: 'web', level: 'bridge' }],
      tests: observed.tests.map((test) => ({
        ...test,
        platform: 'web',
        level: 'bridge',
        pointerWidth: 32,
        binarySha256: identity.binarySha256,
        testBinarySha256: identity.testBinarySha256,
      })),
    };
    const execution: Execution = {
      schemaVersion: 1,
      source: identity.source,
      results: observed.results.map((result) => ({
        ...result,
        attempt,
        platform: 'web',
        binarySha256: identity.binarySha256,
        testBinarySha256: identity.testBinarySha256,
        device: `Node Wasm runtime ${platform()}/${arch()}`,
        os: `${platform()} ${release()} ${arch()}`,
        inputRoute: result.testId.includes('/injected-failures.test.ts:')
          ? 'TypeScript adapter with injected transport failure; no browser or physical input'
          : 'Actual production Rust/Wasm via TypeScript adapter; no browser or physical input',
        artifacts: ['identity.json', 'node-observations.json', 'build.log'],
        environment: unchanged
          ? result.environment
          : {
              valid: false,
              reasons: [...result.environment.reasons, 'Source, Wasm, adapter or tests changed'],
            },
      })),
    };
    validateDiscovery(discovery);
    validateExecution(execution);
    await save('discovery.json', discovery);
    await persistExecution(execution, controller.signal, join(output, 'execution.json'));
    if (
      !execution.results.length ||
      execution.results.some((result) => result.status !== 'passed' || !result.environment.valid)
    )
      throw new Error('Web adapter checks did not pass; see execution.json');
    console.log(`Passed ${execution.results.length} web adapter bridge checks: ${output}`);
  } catch (error) {
    await save('failure.json', { timestamp: new Date().toISOString(), error: String(error) });
    throw error;
  }
});
