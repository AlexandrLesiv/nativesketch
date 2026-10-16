import { resolve, join, isAbsolute } from 'node:path';
import { root, main, command, controller } from './lib/context.ts';
import { sourceIdentity } from './evidence/identity.ts';
import { buildNative } from './android-jni/package-native.ts';
import type { NativeLibrary } from './android-jni/package-native.ts';
import {
  exclusiveDirectory,
  assertStable,
  captureStage,
  publishQualification,
} from './android-jni/session.ts';
import { verifyApk } from './android-jni/package-apk.ts';

/** Provisional additive packaging. Legacy Activity/C++ libraries are intentionally retained. */
await main(async () => {
  const args = process.argv.slice(2);
  if (args.length !== 2 || args[0] !== '--native-source' || !isAbsolute(args[1]))
    throw new Error('Usage: --native-source /absolute/Rust/checkout');
  const nativeRoot = resolve(args[1]),
    ndk = process.env.ANDROID_NDK_HOME;
  if (!ndk || !['darwin', 'linux'].includes(process.platform))
    throw new Error('Requires supported host and explicit ANDROID_NDK_HOME');
  const bin = join(
    ndk,
    `toolchains/llvm/prebuilt/${process.platform === 'darwin' ? 'darwin' : 'linux'}-x86_64/bin`,
  );
  const output = join(
    root,
    'artifacts/evidence/android-jni-package',
    new Date().toISOString().replaceAll(':', '-'),
  );
  await exclusiveDirectory(output);
  const identity = async () => ({
    source: await sourceIdentity(root),
    nativeSource: await sourceIdentity(nativeRoot),
  });
  const initial = await identity();
  const issues: string[] = [];
  const libraries: NativeLibrary[] = [];
  let apk: Awaited<ReturnType<typeof verifyApk>> | null = null;
  const stage = (name: string, work: () => Promise<string>) => captureStage(output, name, work);
  try {
    await buildNative(nativeRoot, bin, stage, libraries);
    // Rebuild both legacy ABIs and package; child output remains bounded and retained on failure.
    await stage('legacy-and-apk', () =>
      command(process.execPath, ['scripts/build.ts', '--platform', 'android'], {
        capture: true,
        timeout: 600_000,
      }),
    );
    apk = await verifyApk(join(root, 'artifacts/NativeSketch-android.apk'), libraries);
  } catch (error) {
    issues.push(String(error));
  }
  let after: Awaited<ReturnType<typeof identity>> | null = null;
  try {
    after = await identity();
    assertStable(initial, after);
  } catch (error) {
    issues.push(String(error));
  }
  await publishQualification(output, controller.signal, issues, () => ({
    'identity.json': {
      initial,
      after,
      passed: issues.length === 0,
      issues,
      libraries,
      apk,
      scope:
        'Additive JNI packaging; legacy Activity retained. No Android execution/UI acceptance.',
    },
  }));
  console.log(`Android JNI package evidence: ${output}`);
  if (issues.length) throw new Error('JNI packaging failed; retained evidence lists issues');
});
