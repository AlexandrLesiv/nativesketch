import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { createHash } from 'node:crypto';
import { join } from 'node:path';
import { root, command, controller } from '../lib/context.ts';
import { fileHash } from '../evidence/identity.ts';
import type { NativeLibrary } from './package-native.ts';
export async function verifyApk(apk: string, libraries: NativeLibrary[]) {
  const before = await fileHash(apk);
  const listing = await command('unzip', ['-Z1', apk], { capture: true });
  const entries = listing
    .split('\n')
    .filter((name) => name.startsWith('lib/') && name.endsWith('.so'))
    .sort();
  const expected = libraries
    .flatMap(({ abi }) =>
      ['libsketch_app_jni.so', 'libsketch_jni.so', 'libsketch_ffi.so'].map(
        (name) => `lib/${abi}/${name}`,
      ),
    )
    .sort();
  if (JSON.stringify(entries) !== JSON.stringify(expected))
    throw new Error('APK native inventory mismatch');
  const hashes = [];
  for (const entry of entries) {
    const { stdout } = await promisify(execFile)('unzip', ['-p', apk, entry], {
      encoding: 'buffer',
      signal: controller.signal,
      maxBuffer: 128 * 1024 * 1024,
      timeout: 15_000,
    });
    const sha256 = createHash('sha256').update(stdout).digest('hex');
    const additive = libraries.find(
      (library) => entry === `lib/${library.abi}/libsketch_app_jni.so`,
    );
    const generated = join(
      root,
      'apps/android/app/build/intermediates/stripped_native_libs/debug/stripDebugDebugSymbols/out',
      entry,
    );
    const expectedHash = additive?.sha256 ?? (await fileHash(generated));
    if (sha256 !== expectedHash) throw new Error(`APK/build identity mismatch: ${entry}`);
    hashes.push({ entry, sha256 });
  }
  for (const library of libraries)
    if ((await fileHash(library.path)) !== library.sha256)
      throw new Error('Staged additive JNI library changed during packaging');
  if ((await fileHash(apk)) !== before) throw new Error('APK changed during inspection');
  return { sha256: before, entries: hashes };
}
