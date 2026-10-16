import { readFile, writeFile } from 'node:fs/promises';
import { join, relative } from 'node:path';
import { platform, release, arch } from 'node:os';
import type { JSONReport } from 'playwright/types/testReporter';
import type { Discovery, Execution } from '../evidence/types.ts';
import { validateDiscovery, validateExecution } from '../evidence/validate.ts';
import { root } from '../lib/context.ts';
import { run } from '../lib/process.ts';
import { sourceIdentity } from '../evidence/identity.ts';
import { prepareOutput } from '../browser-e2e/output.ts';
import { browserSpecs } from '../browser-e2e/results.ts';
import { identity, testId } from './identity.ts';
import { backendEvidence } from './backend.ts';
import {
  failedProcess,
  executionReasons,
  certifiedOutcome,
  processSucceeded,
} from '../browser-e2e/process-result.ts';
import type { ProcessResult } from '../browser-e2e/process-result.ts';

export async function workbenchJourney(options: { output: string; attempt: number }) {
  const { output, attempt } = options;
  const executable = process.env.SKETCH_BROWSER_EXECUTABLE;
  if (!executable) throw new Error('Set SKETCH_BROWSER_EXECUTABLE to installed headless Chromium');
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
    const source = await sourceIdentity(root);
    const build = await logged('build.log', [
      'node_modules/vite/bin/vite.js',
      'build',
      '--config',
      'apps/workbench/vite.config.ts',
    ]);
    if (!processSucceeded(build)) throw new Error('Workbench build failed');
    const before = await identity(executable);
    if (JSON.stringify(source) !== JSON.stringify(before.source))
      throw new Error('Source changed during build');
    const version = await run(executable, ['--version'], {
      cwd: root,
      capture: true,
      timeout: 10_000,
    });
    await save('identity.json', {
      ...before,
      version,
      node: process.version,
      os: `${platform()} ${release()} ${arch()}`,
    });
    await backendEvidence(root, output, source, attempt);
    const cli = [
      'node_modules/@playwright/test/cli.js',
      'test',
      '--config',
      'apps/workbench/e2e/playwright.config.ts',
    ];
    const env = {
      SKETCH_BROWSER_EXECUTABLE: executable,
      SKETCH_WORKBENCH_OUTPUT: join(output, 'test-artifacts'),
    };
    if (
      !processSucceeded(
        await logged('discovery.log', [...cli, '--list'], {
          ...env,
          SKETCH_WORKBENCH_REPORT: join(output, 'enumeration.json'),
        }),
      )
    )
      throw new Error('Workbench discovery failed');
    const read = async (name: string): Promise<JSONReport> =>
      JSON.parse(await readFile(join(output, name), 'utf8'));
    const specs = browserSpecs(await read('enumeration.json'), testId);
    const discovery: Discovery = {
      schemaVersion: 1,
      source,
      scopes: [{ runner: 'playwright', platform: 'workbench', level: 'browser-e2e' }],
      tests: specs.map((spec) => ({
        id: testId(spec),
        runner: 'playwright',
        platform: 'workbench',
        level: 'browser-e2e',
        binarySha256: before.binarySha256,
        testBinarySha256: before.testBinarySha256,
      })),
    };
    validateDiscovery(discovery);
    await save('discovery.json', discovery);
    const completion = await logged('execution.log', cli, {
      ...env,
      SKETCH_WORKBENCH_REPORT: join(output, 'playwright.json'),
    });
    await save('process-result.json', completion);
    const executed = browserSpecs(await read('playwright.json'), testId);
    if (JSON.stringify(executed.map(testId).sort()) !== JSON.stringify(specs.map(testId).sort()))
      throw new Error('Discovery/execution mismatch');
    const reasons = executionReasons(completion, executed);
    if (JSON.stringify(await identity(executable)) !== JSON.stringify(before))
      reasons.push('Source, browser, test code or assets changed during execution');
    const execution: Execution = {
      schemaVersion: 1,
      source,
      results: executed.map((spec) => {
        const outcome = certifiedOutcome(spec, reasons);
        return {
          testId: testId(spec),
          attempt,
          status: outcome.status,
          durationMs: outcome.run.duration,
          timestamp: outcome.run.startTime,
          binarySha256: before.binarySha256,
          testBinarySha256: before.testBinarySha256,
          platform: 'workbench',
          toolchain: `Node ${process.version}; ${version}`,
          device: 'Chromium headless; fixture and actual-service journeys are separately named',
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
    await save('results.json', execution);
    if (!processSucceeded(completion) || execution.results.some((r) => r.status !== 'passed'))
      throw new Error('Workbench acceptance did not pass; see results.json');
    console.log(`Passed ${execution.results.length} workbench browser journeys: ${output}`);
  } catch (error) {
    await save('failure.json', { timestamp: new Date().toISOString(), error: String(error) });
    throw error;
  } finally {
    controller.abort();
    process.removeListener('SIGINT', cancel);
    process.removeListener('SIGTERM', cancel);
  }
}
