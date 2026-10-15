import { cp, mkdir, stat, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { userInfo } from 'node:os';
import { command, root, requireHost } from '../lib/context.mjs';
import { fileHash, sourceIdentity } from '../evidence/identity.mjs';
import { logged, prepareTests, readJSON, testArguments } from './xcode.mjs';
import { enumeratedTests, executionOutcome, testOutcomes } from './xcode-results.mjs';

/** @param {string} output @param {boolean} [execute] @param {number} [attempt] */
export async function macosJourney(output, execute = false, attempt = 1) {
  requireHost('darwin');
  await mkdir(output, { recursive: false });
  const source = await sourceIdentity(root);
  const assertSource = async () => {
    if (JSON.stringify(await sourceIdentity(root)) !== JSON.stringify(source))
      throw new Error('Source changed during E2E preparation/execution; evidence is stale');
  };
  const discovery = { schemaVersion: 1, source, tests: /** @type {any[]} */ ([]) };
  const results = { schemaVersion: 1, source, results: /** @type {any[]} */ ([]) };
  const save = async () => {
    await writeFile(join(output, 'discovery.json'), JSON.stringify(discovery, null, 2));
    await writeFile(join(output, 'results.json'), JSON.stringify(results, null, 2));
  };
  await save();
  try {
    await logged(output, 'app-build.log', process.execPath, [
      'scripts/build.mjs',
      '--platform',
      'macos',
    ]);
    const app = join(output, 'Subject.app');
    await cp(join(root, 'artifacts/NativeSketch.app'), app, { recursive: true });
    const isolation = createHash('sha256')
      .update(root + JSON.stringify(source))
      .digest('hex')
      .slice(0, 24);
    const bundleIdentifier = `dev.nativesketch.e2e.${isolation}`;
    await command('plutil', [
      '-replace',
      'CFBundleIdentifier',
      '-string',
      bundleIdentifier,
      join(app, 'Contents/Info.plist'),
    ]);
    await command('codesign', ['--force', '--sign', '-', app]);
    const binarySha256 = await fileHash(join(app, 'Contents/MacOS/NativeSketch'));
    const { configuration, testBinary } = await prepareTests(output, app);
    const testBinarySha256 = await fileHash(testBinary);
    await assertSource();
    const toolchain = await logged(output, 'toolchain.log', 'xcodebuild', ['-version']);
    const os = await command('sw_vers', [], { capture: true });
    const device = `${await command('sysctl', ['-n', 'hw.model'], { capture: true })}/${process.arch}`;
    await writeFile(
      join(output, 'identity.json'),
      JSON.stringify(
        {
          source,
          bundleIdentifier,
          binarySha256,
          testBinarySha256,
          toolchain,
          os,
          device,
          inputRoute: 'XCUIAutomation mouse/keyboard',
        },
        null,
        2,
      ),
    );
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
    let executionError;
    const started = performance.now();
    try {
      await logged(output, 'execution.log', 'xcodebuild', [
        ...args,
        '-resultBundlePath',
        join(output, 'journey.xcresult'),
      ]);
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
    for (const id of ids) {
      const outcome = outcomes.find((item) => item.testId === id);
      results.results.push({
        testId: id,
        attempt,
        ...executionOutcome(outcome, executionError),
        durationMs: performance.now() - started,
        timestamp: new Date().toISOString(),
        binarySha256,
        testBinarySha256,
        platform: 'macos',
        toolchain,
        device,
        os,
        inputRoute: 'XCUIAutomation mouse/keyboard, no physical pen',
        artifacts: ['execution.log', 'xcresult-tests.json', 'journey.xcresult', 'identity.json'],
      });
    }
    await save();
    if (executionError || results.results.some((item) => item.status !== 'passed'))
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
