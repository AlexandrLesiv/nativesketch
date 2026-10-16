import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { command, root } from '../../../scripts/lib/context.ts';
import { evidence } from './evidence.ts';

if (!['darwin', 'linux'].includes(process.platform)) {
  throw new Error('The C pthread conformance caller currently supports macOS/Linux hosts.');
}
const report = await evidence();
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
  ...['consumer.c', 'support.c', 'worker.c', 'policy.c'].map((name) =>
    join(root, 'crates/sketch-app-ffi/tests', name),
  ),
  join(root, 'target/release/libsketch_app_ffi.a'),
  ...(process.platform === 'darwin' ? ['-framework', 'Security'] : ['-ldl', '-lpthread', '-lm']),
  '-o',
  native,
]);
await report.record(
  'native-c:portable-v2-consumer',
  process.platform === 'darwin' ? 'macos' : 'linux',
  native,
  () => command(native, [], { timeout: 30_000, capture: true }),
);
await command('cargo', [
  'build',
  '--locked',
  '--release',
  '-p',
  'sketch-app-wasm',
  '--target',
  'wasm32-unknown-unknown',
]);
await report.record(
  'wasm-js:portable-v2-conformance',
  'web',
  join(root, 'target/wasm32-unknown-unknown/release/sketch_app_wasm.wasm'),
  () =>
    command(
      process.execPath,
      [
        join(root, 'crates/sketch-app-wasm/tests/conformance.ts'),
        join(root, 'target/wasm32-unknown-unknown/release/sketch_app_wasm.wasm'),
      ],
      { timeout: 30_000, capture: true },
    ),
);
console.log('PASS: portable v2 export consumers (no UI/provider or physical input claim)');
