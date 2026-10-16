import type { DiscoveredTest, Result } from './types.ts';
import { nodeProcess } from './node-process.ts';
import { relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { fileHash } from './identity.ts';
/** Node discovery is combined with execution, using isolated structured runner events. */
export async function executeNode(root: string, files: string[], signal?: AbortSignal) {
  const reporter = fileURLToPath(new URL('./node-reporter.ts', import.meta.url));
  const { output, exitFailed } = await nodeProcess(
    root,
    ['--test', `--test-reporter=${reporter}`, ...files],
    signal,
  );
  signal?.throwIfAborted();

  const tests: DiscoveredTest[] = [];

  const results: Result[] = [];
  let summary = false;
  let suiteFailure = false;
  let summaryFailed = false;
  for (const line of output.trim().split('\n')) {
    const event = JSON.parse(line);
    if (event.type === 'test:summary') {
      summary = true;
      summaryFailed ||= event.data.success !== true;
      continue;
    }
    const data = event.data;
    if (data.details.type === 'suite') {
      if (event.type === 'test:fail') suiteFailure = true;
      continue;
    }
    if (!data.file) throw new Error('Node result lacks file identity');
    if (resolve(root, data.name) === resolve(data.file))
      throw new Error(`Node test file failed before discovery: ${data.file}`);
    const path = relative(root, data.file).replaceAll('\\', '/');
    const id = `node:${path}:${data.name}`;
    const binarySha256 = await fileHash(data.file);
    tests.push({ id, runner: 'node', level: 'unit', platform: 'portable', binarySha256 });
    results.push({
      testId: id,
      attempt: 1,
      status: data.skip || data.todo ? 'skipped' : event.type === 'test:pass' ? 'passed' : 'failed',
      durationMs: data.details.duration_ms,
      timestamp: new Date().toISOString(),
      binarySha256,
      platform: 'portable',
      toolchain: process.version,
      device: `${process.platform}/${process.arch}`,
      environment: { valid: true, reasons: [] },
      artifacts: [],
    });
  }
  if (!summary) throw new Error('Node runner did not finish');
  if (
    suiteFailure ||
    ((exitFailed || summaryFailed) && !results.some((r) => r.status === 'failed'))
  )
    for (const result of results)
      result.environment = {
        valid: false,
        reasons: ['Node suite or hook failed; leaf result does not certify suite completion'],
      };
  signal?.throwIfAborted();
  return { tests, results };
}
