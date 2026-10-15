import { readFile, writeFile, readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { command, root } from '../lib/context.mjs';

export const developer = process.env.DEVELOPER_DIR ?? '/Applications/Xcode.app/Contents/Developer';
/** Capture failures as durable artifacts before propagating. @param {string} output @param {string} name @param {string} executable @param {string[]} args */
export async function logged(output, name, executable, args) {
  try {
    const text = await command(executable, args, {
      capture: true,
      env: { DEVELOPER_DIR: developer },
    });
    await writeFile(join(output, name), text);
    return text;
  } catch (error) {
    await writeFile(join(output, name), String(error));
    throw error;
  }
}
/** @param {string} output @param {string} app */
export async function prepareTests(output, app) {
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
    await command('plutil', ['-convert', 'json', '-o', '-', join(products, original)], {
      capture: true,
    }),
  );
  const target = parsed.NativeSketchUITests;
  target.EnvironmentVariables.NATIVE_SKETCH_APP = app;
  target.EnvironmentVariables.NATIVE_SKETCH_DOCUMENT = join(output, 'drawing.nsketch');
  target.TestLanguage = 'en';
  target.TestRegion = 'US';
  target.UserAttachmentLifetime = 'keepAlways';
  target.SystemAttachmentLifetime = 'keepAlways';
  // Move the run file into its evidence directory with explicit product paths.
  const expanded = JSON.stringify(parsed).replaceAll('__TESTROOT__', products);
  const configuration = join(output, 'configured.xctestrun');
  await writeFile(configuration, expanded);
  await command('plutil', ['-convert', 'xml1', configuration]);
  return {
    configuration,
    testBinary: join(
      products,
      'Debug/NativeSketchUITests-Runner.app/Contents/PlugIns/NativeSketchUITests.xctest/Contents/MacOS/NativeSketchUITests',
    ),
  };
}
/** @param {string} configuration */
export function testArguments(configuration) {
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
/** @param {string} path */
export async function readJSON(path) {
  return JSON.parse(await readFile(path, 'utf8'));
}
