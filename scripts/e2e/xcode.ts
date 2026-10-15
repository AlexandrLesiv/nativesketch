import { readFile, writeFile, readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { command, root } from '../lib/context.ts';
import { prepareStableHost, configureStableHost } from './stable-host.ts';
import { teardownRegression } from './teardown-regression.ts';
import { configureSubjectApps } from './subject-apps.ts';
import type { AppIdentity } from './subject-apps.ts';

export const developer = process.env.DEVELOPER_DIR ?? '/Applications/Xcode.app/Contents/Developer';
/** Capture failures as durable artifacts before propagating. */
export async function logged(
  output: string,
  name: string,
  executable: string,
  args: string[],
  timeout?: number,
) {
  try {
    const text = await command(executable, args, {
      capture: true,
      timeout,
      env: { DEVELOPER_DIR: developer },
    });
    await writeFile(join(output, name), text);
    return text;
  } catch (error) {
    await writeFile(join(output, name), String(error));
    throw error;
  }
}

export async function prepareTests(output: string, app: string, focusApp: AppIdentity) {
  await teardownRegression(output, developer);
  const oracle = join(output, 'oracle-regression');
  await logged(output, 'oracle-build.log', 'xcrun', [
    'swiftc',
    '-module-cache-path',
    join(root, 'artifacts/oracle-module-cache'),
    'apps/macos/UITests/StrokeExpectation.swift',
    'apps/macos/UITests/Regression/OracleRegression.swift',
    '-o',
    oracle,
  ]);
  await logged(output, 'oracle-regression.log', oracle, []);
  const derived = join(root, 'artifacts/e2e-build');
  await logged(output, 'test-build.log', 'xcodebuild', [
    'build-for-testing',
    '-project',
    'apps/macos/UITests/NativeSketchUITests.xcodeproj',
    '-scheme',
    'NativeSketchUITests',
    '-destination',
    `platform=macOS,arch=${process.arch === 'arm64' ? 'arm64' : 'x86_64'}`,
    '-derivedDataPath',
    derived,
  ]);
  const products = join(derived, 'Build/Products');
  const original = (await readdir(products)).find(
    (name) => name.startsWith('NativeSketchUITests_') && name.endsWith('.xctestrun'),
  );
  if (!original) throw new Error('No built xctestrun configuration');
  const parsed = JSON.parse(
    (
      await command('plutil', ['-convert', 'json', '-o', '-', join(products, original)], {
        capture: true,
      })
    ).replaceAll('__TESTROOT__', products),
  );
  const target = parsed.NativeSketchUITests;
  const stable = await prepareStableHost({
    originalHost: target.TestHostPath,
    storage: join(root, 'artifacts/e2e-stable-runner'),
    output,
    toolchain: await command('xcodebuild', ['-version'], {
      capture: true,
      env: { DEVELOPER_DIR: developer },
    }),
    architecture: process.arch,
  });
  configureStableHost(target, stable);
  configureSubjectApps(target.EnvironmentVariables, app, focusApp);
  target.EnvironmentVariables.NATIVE_SKETCH_DOCUMENT = join(output, 'drawing.nsketch');
  target.TestLanguage = 'en';
  target.TestRegion = 'US';
  target.UserAttachmentLifetime = 'keepAlways';
  target.SystemAttachmentLifetime = 'keepAlways';
  // Move the run file into its evidence directory with explicit product paths.
  const expanded = JSON.stringify(parsed);
  const configuration = join(output, 'configured.xctestrun');
  await writeFile(configuration, expanded);
  await command('plutil', ['-convert', 'xml1', configuration]);
  return {
    configuration,
    testBinary: stable.testBinary,
    stable,
  };
}

export function testArguments(configuration: string) {
  return [
    'test-without-building',
    '-xctestrun',
    configuration,
    '-destination',
    `platform=macOS,arch=${process.arch === 'arm64' ? 'arm64' : 'x86_64'}`,
    '-parallel-testing-enabled',
    'NO',
  ];
}

export async function readJSON(path: string) {
  return JSON.parse(await readFile(path, 'utf8'));
}
