import { mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { command, root, requireHost } from '../lib/context.ts';
import { appleEnv, swiftFiles } from './apple.ts';
export async function checkAppleViewers() {
  requireHost('darwin');
  const sdk = String(
    await command('xcrun', ['--sdk', 'macosx', '--show-sdk-path'], {
      env: appleEnv,
      capture: true,
    }),
  );
  const env = { ...appleEnv, SDKROOT: sdk };
  await command('cargo', ['build', '--locked', '--release', '-p', 'sketch-ffi'], { env });
  const directory = join(root, 'artifacts/viewer-check');
  await mkdir(directory, { recursive: true });
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
      join(directory, 'Sample.nsketch'),
    ],
    { env },
  );
  await writeFile(
    join(directory, 'main.swift'),
    'import Foundation\n@main struct Check { @MainActor static func main() async throws { try ViewerReplay.run(); try await SourceReplay.run(); print("APPLE VIEWER REPLAY PASS (host adapter)") } }\n',
  );
  await command(
    'xcrun',
    [
      'swiftc',
      '-D',
      'DEBUG',
      '-swift-version',
      '6',
      '-parse-as-library',
      '-I',
      'apps/apple-bridge/CSketchFFI',
      ...(await swiftFiles('apps/apple-sources')),
      ...(await swiftFiles('apps/apple-viewer')),
      join(directory, 'main.swift'),
      'target/release/libsketch_ffi.a',
      '-framework',
      'Security',
      '-o',
      join(directory, 'check'),
    ],
    { env },
  );
  await command(join(directory, 'check'), [], { env });
}
