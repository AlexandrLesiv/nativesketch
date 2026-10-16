import { cp, readFile, writeFile } from 'node:fs/promises';
import { join, relative } from 'node:path';
import { platform, release, arch } from 'node:os';
import type { JSONReport } from 'playwright/types/testReporter';
import type { Discovery, Execution } from '../evidence/types.ts';
import { validateDiscovery, validateExecution } from '../evidence/validate.ts';
import { root } from '../lib/context.ts';
import { run } from '../lib/process.ts';
import { serve } from '../lib/server.ts';
import { sourceIdentity } from '../evidence/identity.ts';
import { prepareOutput } from './output.ts';
import { browserIdentity, treeHash } from './identity.ts';
import { browserSpecs, browserTestId } from './results.ts';
import {
  failedProcess,
  executionReasons,
  certifiedOutcome,
  processSucceeded,
} from './process-result.ts';
import type { ProcessResult } from './process-result.ts';

export async function browserJourney(options: { output: string; attempt: number }) {
  const { output, attempt } = options;
  const executable = process.env.SKETCH_BROWSER_EXECUTABLE;
  if (!executable)
    throw new Error('Set SKETCH_BROWSER_EXECUTABLE to the installed headless Chromium shell');
  await prepareOutput(root, output);
  const controller = new AbortController();
  const cancel = () => controller.abort();
  process.once('SIGINT', cancel);
  process.once('SIGTERM', cancel);
  const save = (name: string, value: unknown) =>
    writeFile(join(output, name), JSON.stringify(value, null, 2));
  const logged = async (
    name: string,
    args: string[],
    env: NodeJS.ProcessEnv = {},
  ): Promise<ProcessResult> => {
    try {
      const text = await run(process.execPath, args, {
        cwd: root,
        env,
        capture: true,
        timeout: 600_000,
        signal: controller.signal,
      });
      await writeFile(join(output, name), text);
      return { exitCode: 0, signal: null, reason: '' };
    } catch (error) {
      await writeFile(join(output, name), String(error));
      return failedProcess(error);
    }
  };
  try {
    const beforeBuild = await sourceIdentity(root);
    if (!processSucceeded(await logged('build.log', ['scripts/build.ts', '--platform', 'web'])))
      throw new Error('Production build failed; see build.log');
    const identity = await browserIdentity(executable);
    if (JSON.stringify(identity.source) !== JSON.stringify(beforeBuild))
      throw new Error('Source changed while building');
    const browserVersion = await run(executable, ['--version'], {
      cwd: root,
      capture: true,
      timeout: 10_000,
    });
    const served = join(output, 'served', 'nativesketch');
    await cp(join(root, 'artifacts/web'), served, { recursive: true });
    if ((await treeHash(served)) !== identity.binarySha256)
      throw new Error('Served assets differ from production build');
    const server = await serve(join(output, 'served'), 0, controller.signal);
    const address = server.address();
    if (!address || typeof address === 'string') throw new Error('Missing loopback address');
    const baseURL = `http://127.0.0.1:${address.port}/nativesketch/`;
    const env = {
      SKETCH_BROWSER_EXECUTABLE: executable,
      SKETCH_BROWSER_URL: baseURL,
      SKETCH_BROWSER_OUTPUT: output,
      SKETCH_BROWSER_SERVED: served,
    };
    await save('identity.json', {
      ...identity,
      browserVersion,
      node: process.version,
      os: `${platform()} ${release()} ${arch()}`,
      baseURL,
      attempt,
    });
    const cli = [
      'node_modules/@playwright/test/cli.js',
      'test',
      '--config',
      'apps/web/e2e/playwright.config.ts',
    ];
    if (
      !processSucceeded(
        await logged('discovery.log', [...cli, '--list'], {
          ...env,
          SKETCH_BROWSER_REPORT: join(output, 'enumeration.json'),
        }),
      )
    )
      throw new Error('Browser discovery failed');
    const readReport = async (name: string): Promise<JSONReport> =>
      JSON.parse(await readFile(join(output, name), 'utf8'));
    const specs = browserSpecs(await readReport('enumeration.json'));
    const discovery: Discovery = {
      schemaVersion: 1,
      source: identity.source,
      scopes: [{ runner: 'playwright', platform: 'web', level: 'browser-e2e' }],
      tests: specs.map((spec) => ({
        id: browserTestId(spec),
        runner: 'playwright',
        platform: 'web',
        level: 'browser-e2e',
        binarySha256: identity.binarySha256,
        testBinarySha256: identity.testBinarySha256,
      })),
    };
    validateDiscovery(discovery);
    await save('discovery.json', discovery);
    const completion = await logged('execution.log', cli, {
      ...env,
      SKETCH_BROWSER_REPORT: join(output, 'playwright.json'),
    });
    await save('process-result.json', completion);
    const executed = browserSpecs(await readReport('playwright.json'));
    const discoveredIds = discovery.tests.map((test) => test.id).sort();
    if (JSON.stringify(executed.map(browserTestId).sort()) !== JSON.stringify(discoveredIds))
      throw new Error('Discovery/execution mismatch');
    const changed =
      JSON.stringify(await browserIdentity(executable)) !== JSON.stringify(identity) ||
      (await treeHash(served)) !== identity.binarySha256;
    const reasons = executionReasons(completion, executed);
    if (changed) reasons.push('Source, browser, tests or served assets changed during execution');
    const execution: Execution = {
      schemaVersion: 1,
      source: identity.source,
      results: executed.map((spec) => {
        const outcome = certifiedOutcome(spec, reasons);
        return {
          testId: browserTestId(spec),
          attempt,
          status: outcome.status,
          durationMs: outcome.run.duration,
          timestamp: outcome.run.startTime,
          binarySha256: identity.binarySha256,
          testBinarySha256: identity.testBinarySha256,
          platform: 'web',
          toolchain: `Node ${process.version}; ${browserVersion}`,
          device: 'Chromium headless software browser',
          os: `${platform()} ${release()} ${arch()}`,
          inputRoute: 'browser-dispatched mouse and keyboard; not physical pen',
          environment: outcome.environment,
          artifacts: [
            'playwright.json',
            'execution.log',
            'identity.json',
            'process-result.json',
            ...outcome.run.attachments.flatMap((a) => (a.path ? [relative(output, a.path)] : [])),
          ],
          output: outcome.output,
        };
      }),
    };
    validateExecution(execution);
    await save('execution.json', execution);
    if (
      !processSucceeded(completion) ||
      execution.results.some((result) => result.status !== 'passed')
    )
      throw new Error('Browser acceptance did not pass; see execution.json');
    console.log(`Passed ${execution.results.length} browser journeys: ${output}`);
  } catch (error) {
    await save('failure.json', { timestamp: new Date().toISOString(), error: String(error) });
    throw error;
  } finally {
    controller.abort();
    process.removeListener('SIGINT', cancel);
    process.removeListener('SIGTERM', cancel);
  }
}
