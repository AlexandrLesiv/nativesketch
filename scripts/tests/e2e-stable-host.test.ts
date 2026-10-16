import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, writeFile, rm, symlink } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { bundleHash } from '../e2e/bundle-identity.ts';
import { prepareStableHost, configureStableHost, testBundleRelative } from '../e2e/stable-host.ts';
import { fileHash } from '../evidence/identity.ts';

async function fixture() {
  const directory = await mkdtemp(join(tmpdir(), 'native-stable-host-'));
  const originalHost = join(directory, 'Built.app');
  const executable = join(originalHost, testBundleRelative, 'Contents/MacOS/NativeSketchUITests');
  await mkdir(join(originalHost, testBundleRelative, 'Contents/MacOS'), { recursive: true });
  await writeFile(executable, 'first test binary');
  const inspect = async (path: string) => ({
    identifier: 'test.runner',
    requirement: 'fixture-only',
    executableSha256: 'fixture-host',
    treeSha256: await bundleHash(path),
  });
  const options = {
    originalHost,
    storage: join(directory, 'stable'),
    toolchain: 'fixture Xcode',
    architecture: 'fixture arm64',
    inspect,
  };
  const attempt = async (name: string) => {
    const output = join(directory, name);
    await mkdir(output);
    return { ...options, output };
  };
  return { directory, executable, options, attempt };
}

await test('test rebuild updates external bundle without changing the frozen host', async () => {
  const f = await fixture();
  try {
    const first = await prepareStableHost(await f.attempt('first'));
    const hostHash = await bundleHash(first.host);
    await writeFile(f.executable, 'changed test binary');
    const second = await prepareStableHost(await f.attempt('second'));
    assert.equal(first.host, second.host);
    assert.equal(hostHash, await bundleHash(second.host));
    assert.notEqual(first.testBinarySha256, second.testBinarySha256);
    assert.equal(second.testBinarySha256, await fileHash(f.executable));
    assert.equal(await readFile(first.testBinary, 'utf8'), 'first test binary');
    assert.deepEqual(first.hostIdentity, second.hostIdentity);
  } finally {
    await rm(f.directory, { recursive: true, force: true });
  }
});

await test('host mutations and toolchain changes fail closed with durable evidence', async () => {
  const f = await fixture();
  try {
    const first = await prepareStableHost(await f.attempt('first'));
    const incompatible = { ...(await f.attempt('upgrade')), toolchain: 'next Xcode' };
    await assert.rejects(prepareStableHost(incompatible), /explicit host renewal/);
    await writeFile(join(first.host, 'unexpected'), 'mutation');
    const changed = await f.attempt('mutation');
    await assert.rejects(prepareStableHost(changed), /explicit host renewal/);
    assert.match(
      await readFile(join(changed.output, 'runner-preparation-failure.json'), 'utf8'),
      /mutated/,
    );
    assert.equal(await readFile(join(first.host, 'unexpected'), 'utf8'), 'mutation');
  } finally {
    await rm(f.directory, { recursive: true, force: true });
  }
});

await test('partial bootstrap and existing external destination are never replaced', async () => {
  const f = await fixture();
  try {
    const firstOptions = await f.attempt('first');
    await prepareStableHost(firstOptions);
    await assert.rejects(prepareStableHost(firstOptions), /exist/i);
    await rm(join(f.options.storage, 'host-v1/identity.json'));
    await assert.rejects(prepareStableHost(await f.attempt('partial')), /Partial stable/);
  } finally {
    await rm(f.directory, { recursive: true, force: true });
  }
});

await test('bundle identity includes symlink destinations without following them', async () => {
  const f = await fixture();
  try {
    const path = join(f.options.originalHost, 'link');
    await symlink('/nonexistent/a', path);
    const first = await bundleHash(f.options.originalHost);
    await rm(path);
    await symlink('/nonexistent/b', path);
    assert.notEqual(first, await bundleHash(f.options.originalHost));
    const alias = join(f.directory, 'alias.app');
    await symlink(f.options.originalHost, alias);
    await assert.rejects(bundleHash(alias), /real directory/);
  } finally {
    await rm(f.directory, { recursive: true, force: true });
  }
});

await test('configuration uses external current tests and preserves generated environment', () => {
  const target = {
    TestHostPath: '/built/Runner.app',
    TestBundlePath: `__TESTHOST__/${testBundleRelative}`,
    DependentProductPaths: [
      '/built/Runner.app',
      `/built/Runner.app/${testBundleRelative}`,
      '/other',
    ],
    EnvironmentVariables: { DYLD_FRAMEWORK_PATH: '__TESTHOST__/Frameworks:__TESTBUNDLE__' },
  };
  const prepared = {
    host: '/frozen/Runner.app',
    testBundle: '/attempt/current.xctest',
    testBinarySha256: 'a'.repeat(64),
  };
  configureStableHost(target, prepared);
  assert.equal(target.TestBundlePath, prepared.testBundle);
  assert.deepEqual(target.DependentProductPaths, [prepared.host, prepared.testBundle, '/other']);
  assert.equal(
    target.EnvironmentVariables.DYLD_FRAMEWORK_PATH,
    '__TESTHOST__/Frameworks:__TESTBUNDLE__',
  );
  const stale = {
    ...target,
    TestHostPath: '/built/Runner.app',
    EnvironmentVariables: {
      BAD: `/built/Runner.app/${testBundleRelative}/Contents/MacOS/NativeSketchUITests`,
    },
  };
  assert.throws(() => configureStableHost(stale, prepared), /Stale embedded/);
});
