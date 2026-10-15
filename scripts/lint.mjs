import { command, main, root, requireHost } from './lib/context.mjs';
import { options } from './lib/options.mjs';
import { checkTooling } from './lib/tooling.mjs';
import { join } from 'node:path';
await main(async () => {
  const args = options(process.argv.slice(2));
  switch (args.platform) {
    case 'rust':
      await command('cargo', ['fmt', '--all', '--', '--check']);
      await command('cargo', [
        'clippy',
        '--workspace',
        '--all-targets',
        '--locked',
        '--',
        '-D',
        'warnings',
      ]);
      break;
    case 'web':
      await checkTooling();
      await command(process.execPath, [
        'node_modules/typescript/bin/tsc',
        '-p',
        'apps/web',
        '--noEmit',
      ]);
      break;
    case 'swift':
      requireHost('darwin');
      await command('xcrun', [
        'swift-format',
        'lint',
        '--strict',
        '--recursive',
        ...['macos/Sources', 'ios/Sources', 'apple-sources', 'apple-viewer', 'tvos', 'watchos'].map(
          (p) => join(root, 'apps', p),
        ),
      ]);
      break;
    case 'windows':
      requireHost('win32');
      await command('dotnet', [
        'format',
        'apps/windows/NativeSketch.Windows.csproj',
        '--verify-no-changes',
      ]);
      break;
    case 'android':
      await command(process.env.GRADLE ?? 'gradle', [
        '-p',
        'apps/android',
        '--no-daemon',
        'lintDebug',
      ]);
      break;
    default:
      throw new Error('Specify --platform rust|web|swift|windows|android');
  }
});
