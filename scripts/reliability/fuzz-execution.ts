import { join } from 'node:path';
import { release } from 'node:os';
import type { Execution, Source } from '../evidence/types.ts';
import type { FuzzCommand, buildFuzzTargets } from './fuzz-targets.ts';
import { sourceIdentity } from '../evidence/identity.ts';
import { ProcessFailure } from '../lib/process-failure.ts';
import { controller } from '../lib/context.ts';
import { requireUnchanged } from './validation.ts';
import { requireFuzzOutput } from './fuzz-proof.ts';
import { seedCorpus } from './fuzz-seeds.ts';
import { fuzzExecution } from './fuzz-report.ts';

export async function executeFuzzTargets(
  root: string,
  output: string,
  source: Source,
  rustc: string,
  attempt: number,
  targets: Awaited<ReturnType<typeof buildFuzzTargets>>,
  command: FuzzCommand,
) {
  const execution: Execution = { schemaVersion: 1, source, results: [] };
  const results = [];
  const failures: Error[] = [];
  for (const { name: target, binary, binarySha256, core } of targets) {
    requireUnchanged(source, await sourceIdentity(root));
    controller.signal.throwIfAborted();
    const corpus = await seedCorpus(root, output, target);
    const started = Date.now();
    const result = {
      testId: `libfuzzer:sketch-core:${target}`,
      attempt,
      status: 'failed',
      durationMs: 0,
      timestamp: new Date().toISOString(),
      binarySha256,
      platform: 'portable',
      toolchain: rustc,
      device: `macOS arm64 ${release()}`,
      environment: { valid: true, reasons: [] as string[] },
      artifacts: [`${target}.log`, `${target}-build.log`, 'identity.json', 'tools.json'],
    };
    try {
      const text = await command(
        target,
        binary,
        [
          corpus,
          '-seed=2601016',
          '-runs=25000',
          '-max_total_time=30',
          '-timeout=5',
          '-rss_limit_mb=1024',
          '-print_final_stats=1',
          `-max_len=${target === 'document_codec' ? 65536 : 1024}`,
          `-artifact_prefix=${join(output, 'crashes', `${target}-`)}`,
        ],
        60_000,
      );
      results.push({ target, ...requireFuzzOutput(text), binarySha256, core });
      requireUnchanged(source, await sourceIdentity(root));
      controller.signal.throwIfAborted();
      result.status = 'passed';
    } catch (error) {
      result.environment =
        error instanceof ProcessFailure && !error.reason
          ? { valid: true, reasons: [] }
          : { valid: false, reasons: [String(error)] };
      // Independent targets still run after a real child failure. Environmental
      // failures, cancellation and source mutation stop the lane immediately.
      requireUnchanged(source, await sourceIdentity(root));
      controller.signal.throwIfAborted();
      if (!(error instanceof ProcessFailure) || error.reason) throw error;
      failures.push(error);
    } finally {
      result.durationMs = Date.now() - started;
      execution.results.push(result);
      await fuzzExecution(output, execution);
    }
  }
  if (failures.length) throw new AggregateError(failures, failures.map(String).join('\n'));
  return results;
}
