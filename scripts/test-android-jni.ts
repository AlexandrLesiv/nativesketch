import { rm, writeFile, stat, cp, readFile } from 'node:fs/promises';
import { join, resolve, isAbsolute } from 'node:path';
import { root, command, main, controller } from './lib/context.ts';
import { fileHash, sourceIdentity } from './evidence/identity.ts';
import { validateDescriptors, validateExports } from './android-jni/contracts.ts';
import { harvest, normalizedRecords } from './android-jni/evidence.ts';
import { exclusiveDirectory, assertStable, publishQualification } from './android-jni/session.ts';
import { classIdentity } from './android-jni/identity.ts';

/** Real host JVM only; evidence directories are exclusive and failed cases are retained. */
await main(async () => {
  const args = process.argv.slice(2),
    options = new Map<string, string>();
  const allowed = new Set(['--library', '--probe-library', '--native-source', '--output']);
  for (let index = 0; index < args.length; index += 2) {
    const key = args[index],
      value = args[index + 1];
    if (!key || !allowed.has(key) || !value || options.has(key))
      throw new Error('Invalid/duplicate JNI runner option');
    options.set(key, value);
  }
  const required = (key: string) => {
    const value = options.get(key);
    if (!value || !isAbsolute(value)) throw new Error(`${key} requires an absolute path`);
    return value;
  };
  const library = required('--library'),
    probe = required('--probe-library'),
    nativeRoot = required('--native-source');
  for (const path of [library, probe])
    if (!(await stat(path)).isFile()) throw new Error(`Not a library: ${path}`);
  if (!['darwin', 'linux'].includes(process.platform))
    throw new Error('Host JNI runner requires macOS/Linux');
  const output = resolve(
    options.get('--output') ??
      join(root, 'artifacts/evidence/android-jni', new Date().toISOString().replaceAll(':', '-')),
  );
  await exclusiveDirectory(output);
  const identity = async () => ({
    source: await sourceIdentity(root),
    nativeSource: await sourceIdentity(nativeRoot),
    librarySha256: await fileHash(library),
    probeSha256: await fileHash(probe),
  });
  const initial = await identity();
  await rm(join(root, 'apps/android/app/build/jni-discovery.txt'), { force: true });
  const javaHome = process.env.JAVA_HOME;
  if (!javaHome) throw new Error('JAVA_HOME must identify the actual host JVM');
  const jvm = await command(join(javaHome, 'bin/java'), ['-version'], { capture: true });
  const gradle = process.env.GRADLE ?? 'gradle',
    logs: string[] = [],
    issues: string[] = [];
  const lanes = ['portableJniHostTest', 'portableJniPendingExceptionTest'];
  for (const lane of lanes)
    await rm(join(root, 'apps/android/app/build/test-results', lane), {
      recursive: true,
      force: true,
    });
  try {
    for (const [name, path, isProbe] of [
      ['production', library, false],
      ['probe', probe, true],
    ] as const) {
      const exports = await command(
        'nm',
        [...(process.platform === 'darwin' ? ['-gU'] : ['-D', '--defined-only']), path],
        { capture: true },
      );
      await writeFile(join(output, `${name}-exports.txt`), exports);
      validateExports(exports, isProbe);
    }
    logs.push(
      await command(
        gradle,
        ['-p', 'apps/android', '--no-daemon', '--console=plain', ':app:portableJniDiscover'],
        { capture: true, timeout: 180_000 },
      ),
    );
    const declaration = join(
      root,
      'apps/android/app/build/intermediates/built_in_kotlinc/debug/compileDebugKotlin/classes/dev/nativesketch/portable/PortableJniV1.class',
    );
    const javap = await command(join(javaHome, 'bin/javap'), ['-p', '-s', declaration], {
      capture: true,
    });
    await writeFile(join(output, 'javap.txt'), javap);
    validateDescriptors(javap);
    logs.push(
      await command(
        gradle,
        [
          '-p',
          'apps/android',
          '--no-daemon',
          '--console=plain',
          ...lanes.map((lane) => `:app:${lane}`),
          `-PportableJniLibrary=${library}`,
          `-PportableJniProbeLibrary=${probe}`,
        ],
        { capture: true, timeout: 180_000 },
      ),
    );
  } catch (error) {
    issues.push(String(error));
    logs.push(String(error));
  }
  await writeFile(join(output, 'gradle.log'), logs.join('\n'));
  let discovered: string[] = [];
  try {
    const raw = await readFile(join(root, 'apps/android/app/build/jni-discovery.txt'), 'utf8');
    await writeFile(join(output, 'compiled-discovery.txt'), raw);
    discovered = raw.trim().split('\n');
    if (
      !discovered.length ||
      discovered.some((id) => !/^dev\.nativesketch\.portable\.[A-Za-z0-9]+:[A-Za-z0-9]+$/.test(id))
    )
      throw new Error('Invalid compiled discovery');
  } catch (error) {
    issues.push(String(error));
  }
  const data = await harvest(root, output, discovered);
  issues.push(...data.issues);
  let after: Awaited<ReturnType<typeof identity>> | null = null;
  let jvmExecutableSha256: string | null = null;
  let classes: Awaited<ReturnType<typeof classIdentity>> | null = null;
  try {
    after = await identity();
    assertStable(initial, after);
    classes = await classIdentity(root);
    jvmExecutableSha256 = await fileHash(join(javaHome, 'bin/java'));
    await cp(
      join(root, 'apps/android/app/build/jni-host-classpath.txt'),
      join(output, 'classpath.txt'),
    );
  } catch (error) {
    issues.push(String(error));
  }
  await publishQualification(output, controller.signal, issues, () => {
    const records = normalizedRecords(
      data,
      initial.source,
      {
        production: initial.librarySha256,
        probe: initial.probeSha256,
        classes: classes?.sha256 ?? null,
      },
      jvm,
      issues,
    );
    return {
      'identity.json': {
        initial,
        after,
        checksPassed: issues.length === 0,
        certification: 'execution.json is the authoritative qualification',
        issues,
        jvm,
        jvmExecutableSha256,
        node: process.version,
        platform: process.platform,
        architecture: process.arch,
        classes,
        scope: 'Host JVM/JNI only; Android UI/provider acceptance pending',
      },
      'discovery.json': records.discovery,
      'execution.json': records.execution,
    };
  });
  const passed = issues.length === 0;
  console.log(`JNI host evidence: ${output}; passed=${passed}`);
  if (!passed)
    throw new Error('JNI host qualification failed; retained evidence lists every issue');
});
