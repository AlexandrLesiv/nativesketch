import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { command, root } from '../../../scripts/lib/context.ts';

if (!['darwin', 'linux'].includes(process.platform)) {
  throw new Error('The C pthread conformance caller currently supports macOS/Linux hosts.');
}
const revision = (await command('git', ['rev-parse', 'HEAD'], { capture: true })).trim();
const changes = await command('git', ['status', '--porcelain'], { capture: true });
console.log(
  JSON.stringify({
    revision,
    dirty: changes.length > 0,
    platform: process.platform,
    arch: process.arch,
  }),
);
await command('cargo', ['build', '--locked', '--release', '-p', 'sketch-app-ffi']);
const output = join(root, 'artifacts/portable-exports');
await mkdir(output, { recursive: true });
const native = join(output, 'native-v2-consumer');
await command('clang', [
  '-std=c11',
  '-O2',
  '-Wall',
  '-Wextra',
  '-Werror',
  '-I',
  join(root, 'crates/sketch-app-ffi/include'),
  ...['consumer.c', 'support.c', 'worker.c'].map((name) =>
    join(root, 'crates/sketch-app-ffi/tests', name),
  ),
  join(root, 'target/release/libsketch_app_ffi.a'),
  ...(process.platform === 'darwin' ? ['-framework', 'Security'] : ['-ldl', '-lpthread', '-lm']),
  '-o',
  native,
]);
await command(native, [], { timeout: 30_000 });
await command('cargo', [
  'build',
  '--locked',
  '--release',
  '-p',
  'sketch-app-wasm',
  '--target',
  'wasm32-unknown-unknown',
]);
await command(
  process.execPath,
  [
    join(root, 'crates/sketch-app-wasm/tests/conformance.ts'),
    join(root, 'target/wasm32-unknown-unknown/release/sketch_app_wasm.wasm'),
  ],
  { timeout: 30_000 },
);
console.log('PASS: portable v2 export consumers (no UI/provider or physical input claim)');
