import { mkdir, copyFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { setTimeout } from 'node:timers/promises';
import { command, root, controller } from '../lib/context.mjs';
const sdk = process.env.ANDROID_HOME ?? join(homedir(), 'Library/Android/sdk');
export async function buildAndroid() {
  if (!['darwin', 'linux'].includes(process.platform))
    throw new Error('Unsupported NDK build host');
  const ndk = process.env.ANDROID_NDK_HOME ?? '/private/tmp/native-sketch-sdk/android-ndk-r30';
  const bin = join(
    ndk,
    `toolchains/llvm/prebuilt/${process.platform === 'darwin' ? 'darwin' : 'linux'}-x86_64/bin`,
  );
  const env = {
    ANDROID_HOME: sdk,
    JAVA_HOME:
      process.env.JAVA_HOME ?? '/Applications/Android Studio.app/Contents/jbr/Contents/Home',
    CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER: join(bin, 'aarch64-linux-android26-clang'),
    CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER: join(bin, 'x86_64-linux-android26-clang'),
  };
  for (const [target, abi] of [
    ['aarch64-linux-android', 'arm64-v8a'],
    ['x86_64-linux-android', 'x86_64'],
  ]) {
    await command(
      'cargo',
      ['build', '--locked', '-p', 'sketch-ffi', '--release', '--target', target],
      { env },
    );
    const libs = join(root, 'apps/android/app/src/main/jniLibs', abi);
    await mkdir(libs, { recursive: true });
    await copyFile(
      join(root, `target/${target}/release/libsketch_ffi.so`),
      join(libs, 'libsketch_ffi.so'),
    );
    await command(
      join(bin, `${target}26-clang++`),
      [
        '-std=c++17',
        '-shared',
        '-fPIC',
        '-static-libstdc++',
        '-Icrates/sketch-ffi/include',
        'apps/android/app/src/main/cpp/bridge.cpp',
        `-L${libs}`,
        '-lsketch_ffi',
        '-o',
        join(libs, 'libsketch_jni.so'),
      ],
      { env },
    );
  }
  await command(
    process.env.GRADLE ?? 'gradle',
    ['-p', 'apps/android', '--no-daemon', 'assembleDebug'],
    { env },
  );
  await mkdir(join(root, 'artifacts'), { recursive: true });
  await copyFile(
    join(root, 'apps/android/app/build/outputs/apk/debug/app-debug.apk'),
    join(root, 'artifacts/NativeSketch-android.apk'),
  );
}
export async function checkAndroid() {
  if (!process.env.ANDROID_SERIAL)
    throw new Error('Set ANDROID_SERIAL to the verified disposable test device');
  const adb = join(sdk, 'platform-tools', process.platform === 'win32' ? 'adb.exe' : 'adb');
  await command(adb, ['install', '-r', join(root, 'artifacts/NativeSketch-android.apk')]);
  await command(adb, ['logcat', '-c']);
  let launched = false;
  for (let attempt = 0; attempt < 6; attempt++) {
    const output = String(
      await command(
        adb,
        [
          'shell',
          'am',
          'start',
          '-S',
          '-W',
          '-n',
          'dev.nativesketch/.MainActivity',
          '--ez',
          'replay',
          'true',
        ],
        { capture: true },
      ),
    );
    console.log(output);
    if (
      output.includes('Status: ok') &&
      !/Error:|Activity not started|PackageUpdateActivity/.test(output)
    ) {
      launched = true;
      break;
    }
    if (/Error:/.test(output) || !/Activity not started|PackageUpdateActivity/.test(output))
      throw new Error('Replay activity failed to start');
    await setTimeout(2000, undefined, { signal: controller.signal });
  }
  if (!launched) throw new Error('Replay activity did not start after bounded wait');
  for (let attempt = 0; attempt < 30; attempt++) {
    const output = String(
      await command(adb, ['logcat', '-d', '-s', 'NativeSketchReplay:I', 'AndroidRuntime:E'], {
        capture: true,
      }),
    );
    if (output.includes('FAIL: Android replay')) throw new Error(output);
    if (output.includes('PASS: JNI')) {
      console.log(output);
      return;
    }
    await setTimeout(1000, undefined, { signal: controller.signal });
  }
  throw new Error('Android replay timed out');
}
