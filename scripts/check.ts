import { checkTooling } from './lib/tooling.ts';
import { command, main } from './lib/context.ts';
import { options } from './lib/options.ts';
import { readBoard } from './lib/board.ts';
import { root } from './lib/context.ts';
import { buildMacos, checkMacosRenderPolicy } from './platforms/macos.ts';
import { checkNativeFfi } from './platforms/native-ffi.ts';
import { checkLegacyFfi } from './platforms/legacy-ffi.ts';
import { checkWeb, checkWasm } from './platforms/web.ts';
import { checkLinux, checkWindows } from './platforms/desktop.ts';
import { checkAndroid } from './platforms/android.ts';
import { checkAppleViewers } from './platforms/apple-viewers.ts';
import { checkSwift } from './quality/swift.ts';
await main(async () => {
  const args = options(process.argv.slice(2));
  const routes = {
    web: checkWeb,
    wasm: checkWasm,
    linux: checkLinux,
    windows: checkWindows,
    android: checkAndroid,
    'apple-viewers': checkAppleViewers,
  };
  if (args.platform) {
    const route = routes[args.platform as keyof typeof routes];
    if (!route) throw new Error(`Unsupported check platform ${args.platform}`);
    await route();
    return;
  }
  await readBoard(root);
  await checkTooling();
  await command(process.execPath, ['scripts/quality/check.ts']);
  await command('cargo', ['fmt', '--all', '--', '--check']);
  await command('cargo', ['fmt', '--manifest-path', 'fuzz/Cargo.toml', '--', '--check']);
  await command('cargo', [
    'clippy',
    '--workspace',
    '--all-targets',
    '--locked',
    '--',
    '-D',
    'warnings',
  ]);
  await command('cargo', ['test', '--workspace', '--locked']);
  if (!args.core && process.platform === 'darwin') {
    await checkSwift();
    await checkLegacyFfi();
    await checkNativeFfi();
    const app = await buildMacos();
    await command(`${app}/Contents/MacOS/NativeSketch`, ['--smoke-test']);
    await checkMacosRenderPolicy();
  } else console.log('SKIP: macOS build and ABI smoke check');
  console.log(
    'PASS: applicable integration checks; native interaction and physical pen checks are separate',
  );
});
