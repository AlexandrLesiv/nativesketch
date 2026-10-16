import { join } from 'node:path';
import { writeFile } from 'node:fs/promises';
import { logged } from './xcode.ts';
import { completedTestFailures } from './execution-classification.ts';
import type { TestOutcome } from './xcode-results.ts';

/** Preserve the command failure; only its effect on individual evidence is classified. */
export async function classifyXcodeExit(
  output: string,
  error: unknown,
  outcomes: TestOutcome[],
  selected: string[],
): Promise<unknown> {
  if (!error) return undefined;
  try {
    const bundle = join(output, 'journey.xcresult');
    const invocation = await logged(output, 'xcresult-invocation.json', 'xcrun', [
      'xcresulttool',
      'get',
      'object',
      '--legacy',
      '--path',
      bundle,
      '--format',
      'json',
    ]);
    const summary = await logged(output, 'xcresult-summary.json', 'xcrun', [
      'xcresulttool',
      'get',
      'test-results',
      'summary',
      '--path',
      bundle,
      '--format',
      'json',
    ]);
    const classification = completedTestFailures(
      error,
      JSON.parse(invocation),
      JSON.parse(summary),
      outcomes,
      selected,
    );
    await writeFile(
      join(output, 'execution-classification.json'),
      JSON.stringify(classification, null, 2),
    );
    return undefined;
  } catch (failure) {
    await writeFile(
      join(output, 'execution-classification.json'),
      JSON.stringify(
        {
          kind: 'infrastructure-failure',
          reason: String(failure),
          originalError: error instanceof Error ? error.message : JSON.stringify(error),
        },
        null,
        2,
      ),
    );
    return failure;
  }
}
