import { cp, lstat, mkdir, readFile, realpath, rename, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { randomUUID } from 'node:crypto';
import { isDeepStrictEqual } from 'node:util';
import { inspectRunner, bundleHash } from './bundle-identity.ts';
import { fileHash } from '../evidence/identity.ts';

export const testBundleRelative = 'Contents/PlugIns/NativeSketchUITests.xctest';
const testExecutableRelative = 'Contents/MacOS/NativeSketchUITests';
type Options = {
  originalHost: string;
  storage: string;
  output: string;
  toolchain: string;
  architecture: string;
  inspect?: typeof inspectRunner;
};

/** Freeze the generated host once; never replace an authorized host on rebuild. */
export async function prepareStableHost(options: Options) {
  try {
    return await prepare(options);
  } catch (error) {
    await writeFile(
      join(options.output, 'runner-preparation-failure.json'),
      JSON.stringify(
        {
          originalHost: options.originalHost,
          storage: options.storage,
          toolchain: options.toolchain,
          architecture: options.architecture,
          reason: String(error),
        },
        null,
        2,
      ),
    );
    throw error;
  }
}

async function prepare(options: Options) {
  const inspect = options.inspect ?? inspectRunner;
  const container = join(options.storage, 'host-v1');
  const host = join(container, 'NativeSketchUITests-Runner.app');
  const manifestPath = join(container, 'identity.json');
  const identity = async (path: string) => ({
    schemaVersion: 1,
    toolchain: options.toolchain,
    architecture: options.architecture,
    ...(await inspect(path)),
  });
  let existing: unknown;
  try {
    existing = JSON.parse(await readFile(manifestPath, 'utf8'));
  } catch (error) {
    if (!(error instanceof Error && 'code' in error && error.code === 'ENOENT')) throw error;
    try {
      await lstat(container);
      throw new Error('Partial stable XCTest host exists; explicit host renewal required');
    } catch (absence) {
      if (!(absence instanceof Error && 'code' in absence && absence.code === 'ENOENT'))
        throw absence;
    }
    await mkdir(options.storage, { recursive: true });
    const staging = join(options.storage, `bootstrap-${randomUUID()}`);
    await mkdir(staging);
    const stagedHost = join(staging, 'NativeSketchUITests-Runner.app');
    await cp(options.originalHost, stagedHost, { recursive: true, verbatimSymlinks: true });
    existing = await identity(stagedHost);
    await writeFile(join(staging, 'identity.json'), JSON.stringify(existing, null, 2));
    // Existing/partial containers and races fail rather than overwriting an authorized host.
    await rename(staging, container);
  }
  const observed = await identity(host);
  await writeFile(
    join(options.output, 'runner-host-inspection.json'),
    JSON.stringify({ expected: existing, observed, host }, null, 2),
  );
  if (!isDeepStrictEqual(existing, observed))
    throw new Error(
      'Stable XCTest host mutated or toolchain/architecture changed; explicit host renewal required',
    );
  const bundle = join(options.output, 'NativeSketchUITests.xctest');
  await cp(join(options.originalHost, testBundleRelative), bundle, {
    recursive: true,
    errorOnExist: true,
    force: false,
    verbatimSymlinks: true,
  });
  const testBundle = await realpath(bundle);
  const testBinary = join(testBundle, testExecutableRelative);
  const result = {
    host: await realpath(host),
    testBundle,
    testBinary,
    testBinarySha256: await fileHash(testBinary),
    testBundleSha256: await bundleHash(testBundle),
    hostIdentity: existing,
  };
  await writeFile(join(options.output, 'runner-identity.json'), JSON.stringify(result, null, 2));
  return result;
}

type TestTarget = {
  TestHostPath: string;
  TestBundlePath: string;
  DependentProductPaths?: string[];
  EnvironmentVariables: Record<string, string>;
};

/** Preserve all other Xcode-generated variables, flags and signing metadata. */
export function configureStableHost(
  target: TestTarget,
  prepared: {
    host: string;
    testBundle: string;
    testBinarySha256: string;
  },
) {
  const previous = target.TestHostPath;
  target.TestHostPath = prepared.host;
  target.TestBundlePath = prepared.testBundle;
  target.DependentProductPaths = target.DependentProductPaths?.map((path) => {
    if (path === previous) return prepared.host;
    if (path === `${previous}/${testBundleRelative}`) return prepared.testBundle;
    return path;
  });
  target.EnvironmentVariables.NATIVE_SKETCH_TEST_BUNDLE_PATH = prepared.testBundle;
  target.EnvironmentVariables.NATIVE_SKETCH_TEST_BINARY_SHA256 = prepared.testBinarySha256;
  if (
    JSON.stringify(target).includes(`${previous}/${testBundleRelative}`) ||
    JSON.stringify(target).includes(`__TESTHOST__/${testBundleRelative}`)
  )
    throw new Error('Stale embedded test bundle path remains in XCTest configuration');
}
