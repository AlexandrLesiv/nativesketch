import { checkTooling } from './lib/tooling.mjs';
import { command, main } from './lib/context.mjs';
import { options } from './lib/options.mjs';
import { readBoard } from './lib/board.mjs';
import { root } from './lib/context.mjs';
import { buildMacos } from './platforms/macos.mjs';
import { checkWeb, checkWasm } from './platforms/web.mjs';
import { checkLinux, checkWindows } from './platforms/desktop.mjs';
import { checkAndroid } from './platforms/android.mjs';
import { checkAppleViewers } from './platforms/apple-viewers.mjs';
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
    const route = routes[/** @type {keyof typeof routes} */ (args.platform)];
    if (!route) throw new Error(`Unsupported check platform ${args.platform}`);
    await route();
    return;
  }
  await readBoard(root);
  await checkTooling();
  await command(process.execPath, ['scripts/quality/check.mjs']);
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
  await command('cargo', ['test', '--workspace', '--locked']);
  if (!args.core && process.platform === 'darwin') {
    const app = await buildMacos();
    await command(`${app}/Contents/MacOS/NativeSketch`, ['--smoke-test']);
  } else console.log('SKIP: macOS build and ABI smoke check');
  console.log(
    'PASS: applicable integration checks; native interaction and physical pen checks are separate',
  );
});
