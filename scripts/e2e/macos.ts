import { classifyXcodeExit } from './classify-xcode-exit.ts';
import type { Discovery, Execution } from '../evidence/types.ts';
import { mkdir, stat, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { userInfo } from 'node:os';
import { root, requireHost } from '../lib/context.ts';
import { fileHash, sourceIdentity } from '../evidence/identity.ts';
import { logged, prepareTests, readJSON, testArguments } from './xcode.ts';
import { enumeratedTests, executionOutcome, testOutcomes } from './xcode-results.ts';
import { selectedTests, selectionArguments, nativeInputRoute } from './selection.ts';
import { validateRuntimeProvenance } from './loaded-provenance.ts';
import { captureNativeIdentity } from './native-identity.ts';
import { prepareSubjectApps } from './subject-apps.ts';

export async function macosJourney(
  output: string,
  execute: boolean = false,
  attempt: number = 1,
  requested: string[] = [],
) {
  requireHost('darwin');
  await mkdir(output, { recursive: false });
  const source = await sourceIdentity(root);
  const assertSource = async () => {
    if (JSON.stringify(await sourceIdentity(root)) !== JSON.stringify(source))
      throw new Error('Source changed during E2E preparation/execution; evidence is stale');
  };
  const discovery: Discovery = { schemaVersion: 1, source, tests: [] };
  const results: Execution = { schemaVersion: 1, source, results: [] };
  const save = async () => {
    await writeFile(join(output, 'discovery.json'), JSON.stringify(discovery, null, 2));
    await writeFile(join(output, 'results.json'), JSON.stringify(results, null, 2));
  };
  await save();
  try {
    await logged(output, 'app-build.log', '/usr/bin/env', [
      'SKETCH_NATIVE_TEST_CONTROLS=1',
      process.execPath,
      'scripts/build.ts',
      '--platform',
      'macos',
    ]);
    const { subject, focusApp } = await prepareSubjectApps(output, source);
    const { bundleIdentifier, binarySha256 } = subject;
    const { configuration, testBinary, stable } = await prepareTests(
      output,
      subject.path,
      focusApp,
    );
    const testBinarySha256 = await fileHash(testBinary);
    await assertSource();
    const { toolchain, os, device } = await captureNativeIdentity(output, {
      source,
      bundleIdentifier,
      binarySha256,
      testBinarySha256,
      focusApp,
    });
    if (!execute)
      throw new Error(
        'UI execution deferred: background preparation only; use --interactive in a reserved foreground session. No executable discovery attempted.',
      );
    if ((await stat('/dev/console')).uid !== userInfo().uid)
      throw new Error(
        'XCTest requires this process user to own the active unlocked console; user switching is required. No UI attempt made.',
      );
    const args = testArguments(configuration);
    await logged(output, 'enumeration.log', 'xcodebuild', [
      ...args,
      '-enumerate-tests',
      '-test-enumeration-style',
      'flat',
      '-test-enumeration-format',
      'json',
      '-test-enumeration-output-path',
      join(output, 'enumeration.json'),
    ]);
    const ids = enumeratedTests(await readJSON(join(output, 'enumeration.json')));
    discovery.tests = ids.map((id) => ({
      id,
      runner: 'xctest',
      level: 'native-e2e',
      platform: 'macos',
      binarySha256,
      testBinarySha256,
    }));
    await save();
    const selected = selectedTests(ids, requested);
    await writeFile(join(output, 'selected-tests.json'), JSON.stringify(selected, null, 2));
    let executionError;
    try {
      // Native journeys include real panels, relaunches and worker gates.
      // Bound the whole suite independently of unchanged per-case readiness limits.
      await logged(
        output,
        'execution.log',
        'xcodebuild',
        [
          ...args,
          ...selectionArguments(selected),
          '-resultBundlePath',
          join(output, 'journey.xcresult'),
        ],
        20 * 60_000,
      );
    } catch (error) {
      executionError = error;
    }
    const raw = await logged(output, 'xcresult-tests.json', 'xcrun', [
      'xcresulttool',
      'get',
      'test-results',
      'tests',
      '--path',
      join(output, 'journey.xcresult'),
      '--format',
      'json',
    ]);
    await assertSource();
    const outcomes = testOutcomes(JSON.parse(raw), ids);
    let validationError = await classifyXcodeExit(output, executionError, outcomes, selected);
    try {
      await validateRuntimeProvenance(
        output,
        outcomes.filter((item) => item.status === 'passed').map((item) => item.testId),
        stable,
      );
    } catch (error) {
      validationError = error;
    }
    if (outcomes.some((outcome) => !selected.includes(outcome.testId)))
      throw new Error(
        'XCTest executed a case outside the requested selection; inspect raw results',
      );
    for (const id of selected) {
      const outcome = outcomes.find((item) => item.testId === id);
      results.results.push({
        testId: id,
        attempt,
        ...executionOutcome(outcome, validationError),
        durationMs: outcome?.durationMs ?? 0,
        timestamp: new Date().toISOString(),
        binarySha256,
        testBinarySha256,
        platform: 'macos',
        toolchain,
        device,
        os,
        inputRoute: nativeInputRoute(id),
        artifacts: [
          'execution.log',
          'xcresult-tests.json',
          ...(executionError
            ? ['xcresult-invocation.json', 'xcresult-summary.json', 'execution-classification.json']
            : []),
          'journey.xcresult',
          'identity.json',
          'runner-identity.json',
          'provenance-validation.json',
        ],
      });
    }
    await save();
    if (
      executionError ||
      validationError ||
      results.results.some((item) => item.status !== 'passed')
    )
      throw new Error(
        `Native journey did not pass: ${executionError instanceof Error ? executionError.message : 'see results.json'}`,
      );
  } catch (error) {
    await writeFile(
      join(output, 'blocked.json'),
      JSON.stringify(
        {
          stage: discovery.tests.length ? 'execution' : 'setup',
          reason: String(error),
          timestamp: new Date().toISOString(),
          source,
        },
        null,
        2,
      ),
    );
    await save();
    throw error;
  }
}
