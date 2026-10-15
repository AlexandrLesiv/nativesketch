import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { command, requireHost, root } from '../lib/context.ts';

const crate = 'sketch-native-ffi';
const hooks = ['_sketch_runtime_v1_test_arm', '_sketch_runtime_v1_test_poison'];

async function symbols(profile: 'debug' | 'release') {
  return command('nm', ['-gUj', join(root, `target/${profile}/libsketch_native_ffi.dylib`)], {
    capture: true,
  });
}

/** Real C consumer and feature separation; background only, no native app launch. */
export async function checkNativeFfi() {
  requireHost('darwin');
  const output = join(root, 'artifacts/native-ffi');
  await mkdir(output, { recursive: true });
  await command('cargo', ['build', '--locked', '-p', crate, '--release']);
  const releaseSymbols = (await symbols('release')).split('\n');
  if (hooks.some((hook) => releaseSymbols.includes(hook))) {
    throw new Error('Default release unexpectedly exports native test controls');
  }
  const consumer = join(output, 'consumer');
  await command('clang', [
    '-std=c11',
    '-D_DARWIN_C_SOURCE',
    '-Wall',
    '-Wextra',
    '-Werror',
    '-I',
    join(root, 'crates/sketch-native-ffi/include'),
    join(root, 'crates/sketch-native-ffi/tests/consumer.c'),
    join(root, 'target/release/libsketch_native_ffi.a'),
    '-framework',
    'Security',
    '-o',
    consumer,
  ]);
  await command(consumer, [], { timeout: 30_000 });
  try {
    await command('cargo', [
      'build',
      '--locked',
      '-p',
      crate,
      '--features',
      'native-test-controls',
    ]);
    const debugSymbols = (await symbols('debug')).split('\n');
    if (!hooks.every((hook) => debugSymbols.includes(hook))) {
      throw new Error('Explicit test build is missing its native control entry point');
    }
    await command('cargo', [
      'test',
      '--locked',
      '-p',
      crate,
      '--features',
      'native-test-controls',
      'boundary_tests',
    ]);
    await command('cargo', [
      'test',
      '--locked',
      '-p',
      'sketch-services',
      '--features',
      'native-test-controls',
      '--test',
      'native_controls',
    ]);
  } finally {
    // Restore the ordinary debug library before any subsequent native app link.
    await command('cargo', ['build', '--locked', '-p', crate]);
  }
  console.log('PASS: native C consumer and explicit test-control symbol separation');
}
