import { mkdir, readdir, copyFile } from 'node:fs/promises';
import { join } from 'node:path';
import { command, root, requireHost } from '../lib/context.ts';
import { buildMacos } from './macos.ts';

const hosts: Record<
  string,
  {
    sdk: string[];
    rust: string;
    swift: string;
    name: string;
    id: string;
  }
> = {
  ios: {
    sdk: ['iphonesimulator', 'iphoneos'],
    rust: 'ios',
    swift: 'ios17.0',
    name: 'NativeSketch',
    id: 'dev.nativesketch.ios',
  },
  visionos: {
    sdk: ['xrsimulator', 'xros'],
    rust: 'visionos',
    swift: 'xros1.0',
    name: 'NativeSketch',
    id: 'dev.nativesketch.visionos',
  },
  tvos: {
    sdk: ['appletvsimulator', 'appletvos'],
    rust: 'tvos',
    swift: 'tvos17.0',
    name: 'NativeSketchTV',
    id: 'dev.nativesketch.tv',
  },
  watchos: {
    sdk: ['watchsimulator', 'watchos'],
    rust: 'watchos',
    swift: 'watchos10.0',
    name: 'NativeSketchWatch',
    id: 'dev.nativesketch.ios.watch',
  },
};
export const appleEnv = {
  DEVELOPER_DIR: process.env.DEVELOPER_DIR ?? '/Applications/Xcode.app/Contents/Developer',
};

export async function swiftFiles(path: string) {
  return (await readdir(join(root, path)))
    .filter((p) => p.endsWith('.swift'))
    .sort()
    .map((p) => join(root, path, p));
}

export async function buildApple(platform: string, variant: string = 'simulator'): Promise<string> {
  requireHost('darwin');
  if (!['simulator', 'device'].includes(variant)) throw new Error('Expected simulator or device');
  if (platform === 'apple-family') {
    await buildMacos();
    await buildApple('watchos', variant);
    const phone = await buildApple('ios', variant);
    await command('/usr/bin/ditto', [
      join(root, `artifacts/watchos-${variant}/NativeSketchWatch.app`),
      join(phone, 'Watch/NativeSketchWatch.app'),
    ]);
    await command('/usr/bin/codesign', ['--force', '--sign', '-', phone]);
    await buildApple('tvos', variant);
    await buildApple('visionos', variant);
    return phone;
  }
  const host = hosts[platform];
  if (!host) throw new Error(`Unknown Apple platform ${platform}`);
  const simulator = variant === 'simulator';
  const sdk = host.sdk[simulator ? 0 : 1];
  const target = `aarch64-apple-${host.rust}${simulator ? '-sim' : ''}`;
  const sdkRoot = String(
    await command('xcrun', ['--sdk', sdk, '--show-sdk-path'], { env: appleEnv, capture: true }),
  );
  const env = { ...appleEnv, SDKROOT: sdkRoot };
  await command(
    'cargo',
    ['build', '--locked', '--release', '-p', 'sketch-ffi', '--target', target],
    { env },
  );
  const app = join(root, `artifacts/${platform}-${variant}/${host.name}.app`);
  const cache = join(root, `.build-cache/${platform}-module-cache`);
  await mkdir(app, { recursive: true });
  await mkdir(cache, { recursive: true });
  const editor = ['ios', 'visionos'].includes(platform);
  const sources = editor
    ? await swiftFiles('apps/ios/Sources')
    : [...(await swiftFiles('apps/apple-viewer')), ...(await swiftFiles(`apps/${platform}`))];
  if (platform !== 'visionos') sources.unshift(...(await swiftFiles('apps/apple-sources')));
  const frameworks = editor
    ? ['UIKit', 'UniformTypeIdentifiers', 'Security']
    : [platform === 'watchos' ? 'SwiftUI' : 'UIKit', 'Security'];
  if (['ios', 'watchos'].includes(platform)) frameworks.push('WatchConnectivity');
  const defines = editor ? ['-D', 'NATIVE_SKETCH'] : [];
  if (simulator) defines.push('-D', 'DEBUG');
  else if (!editor) defines.push('-D', 'NATIVE_DEVICE');
  await command(
    'xcrun',
    [
      '--sdk',
      sdk,
      'swiftc',
      ...defines,
      '-swift-version',
      '6',
      '-parse-as-library',
      '-O',
      '-target',
      `arm64-apple-${host.swift}${simulator ? '-simulator' : ''}`,
      '-sdk',
      sdkRoot,
      '-module-cache-path',
      cache,
      '-I',
      'apps/apple-bridge/CSketchFFI',
      ...sources,
      `target/${target}/release/libsketch_ffi.a`,
      ...frameworks.flatMap((f) => ['-framework', f]),
      '-o',
      join(app, host.name),
    ],
    { env },
  );
  await copyFile(join(root, `apps/${platform}/Info.plist`), join(app, 'Info.plist'));
  if (!editor) {
    const macSdk = String(
      await command('xcrun', ['--sdk', 'macosx', '--show-sdk-path'], {
        env: appleEnv,
        capture: true,
      }),
    );
    await command(
      'cargo',
      [
        'run',
        '--locked',
        '--quiet',
        '-p',
        'sketch-ffi',
        '--example',
        'portable-fixture',
        '--',
        join(app, 'Sample.nsketch'),
      ],
      { env: { ...appleEnv, SDKROOT: macSdk } },
    );
  }
  await command('/usr/bin/codesign', ['--force', '--sign', '-', app]);
  return app;
}

export async function runApple(platform: string, device: string, args: string[]) {
  requireHost('darwin');
  if (!device) throw new Error('Pass an exact simulator UUID with --device');
  const host = hosts[platform];
  if (!host) throw new Error('Unsupported Apple platform');
  await command(
    'xcrun',
    ['simctl', 'install', device, join(root, `artifacts/${platform}-simulator/${host.name}.app`)],
    { env: appleEnv },
  );
  const output = String(
    await command('xcrun', ['simctl', 'launch', '--console', device, host.id, ...args], {
      env: appleEnv,
      capture: true,
    }),
  );
  console.log(output);
  if (
    platform === 'watchos' &&
    args.includes('--integration-test') &&
    (!output.includes('APPLE SOURCE REPLAY PASS') || output.includes('APPLE SOURCE REPLAY FAIL'))
  )
    throw new Error('Watch source replay did not complete successfully');
}
