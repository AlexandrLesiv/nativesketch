import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { command, root } from '../lib/context.ts';

export function validateTeardownRegression(status: number | null, log: string, trace: string) {
  if (status !== 1) throw new Error(`Negative XCTest suite exited unexpectedly: ${status}`);
  for (const name of ['testAssertionAbortRunsCleanup', 'testThrowingCleanupPreservesLIFO'])
    if (!log.split('\n').some((line) => line.includes(name) && line.includes('failed')))
      throw new Error(`Missing expected failing test: ${name}`);
  const events: unknown[] = trace
    .trim()
    .split('\n')
    .map((line) => JSON.parse(line));
  const expected = [
    'assertion:body',
    'assertion:cleanup',
    'throwing:body',
    'throwing:inner',
    'throwing:outer',
  ].map((event) => ({ event }));
  if (JSON.stringify(events) !== JSON.stringify(expected))
    throw new Error('Native teardown did not run the exact expected cleanup sequence');
}

/** An intentionally failing headless XCTest suite verifies cleanup after failure. */
export async function teardownRegression(output: string, developer: string, sourceRoot = root) {
  const bundle = join(output, 'NativeTeardownRegression.xctest');
  const contents = join(bundle, 'Contents');
  await mkdir(join(contents, 'MacOS'), { recursive: true });
  await writeFile(
    join(contents, 'Info.plist'),
    `<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>NativeTeardownRegression</string>
<key>CFBundleIdentifier</key><string>dev.nativesketch.TeardownRegression</string>
<key>CFBundlePackageType</key><string>BNDL</string>
</dict></plist>`,
  );
  const frameworks = join(developer, 'Platforms/MacOSX.platform/Developer/Library/Frameworks');
  const libraries = join(developer, 'Platforms/MacOSX.platform/Developer/usr/lib');
  await command(
    'xcrun',
    [
      'swiftc',
      '-emit-library',
      '-swift-version',
      '6',
      '-module-name',
      'NativeTeardownRegression',
      '-F',
      frameworks,
      '-I',
      libraries,
      '-L',
      libraries,
      '-lXCTestSwiftSupport',
      '-Xlinker',
      '-rpath',
      '-Xlinker',
      libraries,
      '-Xlinker',
      '-rpath',
      '-Xlinker',
      frameworks,
      '-module-cache-path',
      join(root, 'artifacts/oracle-module-cache'),
      join(sourceRoot, 'apps/macos/UITests/NativeTestTeardown.swift'),
      join(sourceRoot, 'apps/macos/UITests/Regression/NativeTeardownRegression.swift'),
      '-o',
      join(contents, 'MacOS/NativeTeardownRegression'),
    ],
    { env: { DEVELOPER_DIR: developer } },
  );
  const trace = join(output, 'teardown-regression.jsonl');
  await writeFile(trace, '');
  const result = spawnSync('xcrun', ['xctest', bundle], {
    cwd: root,
    encoding: 'utf8',
    timeout: 30_000,
    maxBuffer: 4_000_000,
    env: { ...process.env, DEVELOPER_DIR: developer, NATIVE_SKETCH_TEARDOWN_TRACE: trace },
  });
  const log = result.stdout + result.stderr;
  await writeFile(join(output, 'teardown-regression.log'), log);
  await writeFile(
    join(output, 'teardown-regression-result.json'),
    JSON.stringify(
      {
        expectedFailureSuite: true,
        exitCode: result.status,
        signal: result.signal,
        error: result.error?.message,
      },
      null,
      2,
    ),
  );
  validateTeardownRegression(result.status, log, await readFile(trace, 'utf8'));
}
