import { mkdir, copyFile } from 'node:fs/promises';
import { join } from 'node:path';
import { command, root, requireHost } from '../lib/context.ts';

export async function buildMacos(profile: string = 'debug') {
  requireHost('darwin');
  if (!['debug', 'release'].includes(profile)) throw new Error('Expected debug or release');
  const testControls = process.env.SKETCH_NATIVE_TEST_CONTROLS === '1';
  if (profile === 'release' && testControls)
    throw new Error('Native test controls require a debug build');
  const env = {
    SKETCH_RUST_PROFILE: profile,
    DEVELOPER_DIR: process.env.DEVELOPER_DIR ?? '/Applications/Xcode.app/Contents/Developer',
    CLANG_MODULE_CACHE_PATH: join(root, '.build-cache/clang'),
    SWIFTPM_MODULECACHE_OVERRIDE: join(root, '.build-cache/swift-modules'),
  };
  for (const path of [env.CLANG_MODULE_CACHE_PATH, env.SWIFTPM_MODULECACHE_OVERRIDE])
    await mkdir(path, { recursive: true });
  await command('cargo', [
    'build',
    '--locked',
    '-p',
    'sketch-native-ffi',
    ...(testControls ? ['--features', 'native-test-controls'] : []),
    ...(profile === 'release' ? ['--release'] : []),
  ]);
  const flags = swiftFlags(profile);
  await command('swift', ['build', ...flags], { env });
  const bin = await command('swift', ['build', ...flags, '--show-bin-path'], {
    env,
    capture: true,
  });
  const app = join(root, 'artifacts/NativeSketch.app');
  await mkdir(join(app, 'Contents/MacOS'), { recursive: true });
  await copyFile(join(String(bin), 'NativeSketch'), join(app, 'Contents/MacOS/NativeSketch'));
  await copyFile(join(root, 'apps/macos/Info.plist'), join(app, 'Contents/Info.plist'));
  await command('/usr/bin/codesign', ['--force', '--sign', '-', app]);
  return app;
}

/** Pure scheduling policy tests; no NSApplication, Metal context or foreground input. */
// GPU-free renderer policy and real Swift/Rust compound-action contracts.
export async function checkMacosRenderPolicy() {
  requireHost('darwin');
  await command(
    'swift',
    ['test', ...swiftFlags('debug'), '--filter', '(Render.*Tests|SessionActionTests)'],
    {
      env: {
        SKETCH_RUST_PROFILE: 'debug',
        DEVELOPER_DIR: process.env.DEVELOPER_DIR ?? '/Applications/Xcode.app/Contents/Developer',
        CLANG_MODULE_CACHE_PATH: join(root, '.build-cache/clang'),
        SWIFTPM_MODULECACHE_OVERRIDE: join(root, '.build-cache/swift-modules'),
      },
    },
  );
}

function swiftFlags(profile: string) {
  return [
    '--package-path',
    join(root, 'apps/macos'),
    '--cache-path',
    join(root, '.build-cache/swift-cache'),
    '--config-path',
    join(root, '.build-cache/swift-config'),
    '--security-path',
    join(root, '.build-cache/swift-security'),
    '--scratch-path',
    join(root, '.build-cache/swift-build'),
    '--manifest-cache',
    'local',
    '--configuration',
    profile,
  ];
}
