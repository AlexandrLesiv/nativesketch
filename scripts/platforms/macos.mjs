import { mkdir, copyFile } from 'node:fs/promises';
import { join } from 'node:path';
import { command, root, requireHost } from '../lib/context.mjs';
/** @param {string} [profile] */
export async function buildMacos(profile = 'debug') {
  requireHost('darwin');
  if (!['debug', 'release'].includes(profile)) throw new Error('Expected debug or release');
  const env = {
    SKETCH_RUST_PROFILE: profile,
    CLANG_MODULE_CACHE_PATH: join(root, '.build-cache/clang'),
    SWIFTPM_MODULECACHE_OVERRIDE: join(root, '.build-cache/swift-modules'),
  };
  for (const path of Object.values(env).slice(1)) await mkdir(path, { recursive: true });
  await command('cargo', [
    'build',
    '--locked',
    '-p',
    'sketch-ffi',
    ...(profile === 'release' ? ['--release'] : []),
  ]);
  const flags = [
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
