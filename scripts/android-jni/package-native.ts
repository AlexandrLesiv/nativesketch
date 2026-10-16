import { mkdir, readFile, rm, copyFile } from 'node:fs/promises';
import { join } from 'node:path';
import { command, root } from '../lib/context.ts';
import { fileHash } from '../evidence/identity.ts';
import { validateExports } from './contracts.ts';
export interface NativeLibrary {
  abi: string;
  path: string;
  sha256: string;
  machine: number;
}
export async function buildNative(
  nativeRoot: string,
  bin: string,
  stage: (name: string, work: () => Promise<string>) => Promise<string>,
  libraries: NativeLibrary[],
): Promise<void> {
  for (const [target, abi, machine] of [
    ['aarch64-linux-android', 'arm64-v8a', 183],
    ['x86_64-linux-android', 'x86_64', 62],
  ] as const) {
    const staging = join(root, 'apps/android/app/src/main/jniLibs', abi);
    await mkdir(staging, { recursive: true });
    for (const name of ['libsketch_app_jni.so', 'libsketch_jni.so', 'libsketch_ffi.so'])
      await rm(join(staging, name), { force: true });
    const targetDir = join(root, 'target/android-portable-jni');
    await stage(`${abi}-build`, () =>
      command(
        'cargo',
        [
          'build',
          '--locked',
          '--release',
          '--manifest-path',
          join(nativeRoot, 'Cargo.toml'),
          '-p',
          'sketch-app-jni',
          '--target',
          target,
        ],
        {
          capture: true,
          timeout: 600_000,
          env: {
            CARGO_TARGET_DIR: targetDir,
            [`CARGO_TARGET_${target.toUpperCase().replaceAll('-', '_')}_LINKER`]: join(
              bin,
              `${target}26-clang`,
            ),
          },
        },
      ),
    );
    const built = join(targetDir, target, 'release/libsketch_app_jni.so');
    const bytes = await readFile(built);
    if (
      bytes.length < 20 ||
      bytes.subarray(0, 4).toString('hex') !== '7f454c46' ||
      bytes[4] !== 2 ||
      bytes[5] !== 1 ||
      bytes.readUInt16LE(18) !== machine
    )
      throw new Error(`Wrong ELF identity for ${abi}`);
    validateExports(
      await stage(`${abi}-symbols`, () =>
        command(join(bin, 'llvm-readelf'), ['--dyn-syms', '--wide', built], { capture: true }),
      ),
      false,
    );
    const path = join(staging, 'libsketch_app_jni.so');
    await copyFile(built, path);
    libraries.push({ abi, path, sha256: await fileHash(path), machine });
  }
}
