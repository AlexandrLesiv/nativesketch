import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, realpath, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { prepareSubjectApps, configureSubjectApps } from '../e2e/subject-apps.ts';
import { fileHash } from '../evidence/identity.ts';

await test('focus helper copies the current isolated build without replacing or sharing its identity', async () => {
  const directory = await realpath(await mkdtemp(join(tmpdir(), 'native-focus-app-')));
  const built = join(directory, 'Built.app');
  const output = join(directory, 'attempt');
  const calls: string[][] = [];
  await mkdir(join(built, 'Contents/MacOS'), { recursive: true });
  await mkdir(output);
  await writeFile(join(built, 'Contents/MacOS/NativeSketch'), 'original build');
  await writeFile(join(built, 'Contents/Info.plist'), 'original.identifier');
  const execute = async (file: string, args: string[]) => {
    calls.push([file, ...args]);
    if (file === 'plutil') {
      const path = args.at(-1);
      assert.ok(path);
      if (args[0] === '-replace') await writeFile(path, args[3]);
      return readFile(path, 'utf8');
    }
    if (args[0] === '--force') {
      const path = args.at(-1);
      assert.ok(path);
      const executable = join(path, 'Contents/MacOS/NativeSketch');
      await writeFile(executable, `${await readFile(executable, 'utf8')} signed`);
    }
    return '';
  };
  try {
    const { subject, focusApp } = await prepareSubjectApps(
      output,
      { revision: 'fixture', dirtyDiffSha256: null },
      built,
      execute,
    );
    assert.equal(focusApp.bundleIdentifier, `${subject.bundleIdentifier}.focus`);
    assert.equal(focusApp.path, join(output, 'FocusSubject.app'));
    assert.equal(focusApp.sourceBinarySha256, subject.binarySha256);
    assert.equal(
      focusApp.binarySha256,
      await fileHash(join(focusApp.path, 'Contents/MacOS/NativeSketch')),
    );
    assert.notEqual(focusApp.binarySha256, subject.binarySha256);
    assert.equal(
      await readFile(join(built, 'Contents/MacOS/NativeSketch'), 'utf8'),
      'original build',
    );
    assert.equal(await readFile(join(built, 'Contents/Info.plist'), 'utf8'), 'original.identifier');
    assert.deepEqual(
      JSON.parse(await readFile(join(output, 'focus-app-identity.json'), 'utf8')),
      focusApp,
    );
    assert.equal(
      calls.filter(([file, arg]) => file === 'codesign' && arg === '--verify').length,
      2,
    );
    await assert.rejects(
      prepareSubjectApps(output, { revision: 'fixture', dirtyDiffSha256: null }, built, execute),
      /exist/i,
    );
    const environment = { EXISTING: 'retained' };
    configureSubjectApps(environment, subject.path, focusApp);
    assert.deepEqual(environment, {
      EXISTING: 'retained',
      NATIVE_SKETCH_APP: subject.path,
      NATIVE_SKETCH_FOCUS_APP: focusApp.path,
      NATIVE_SKETCH_FOCUS_BINARY_SHA256: focusApp.binarySha256,
    });
    const invalidOutput = join(directory, 'mismatched-identity');
    await mkdir(invalidOutput);
    const inconsistentIdentity = async (file: string, args: string[]) =>
      file === 'plutil' && args[0] === '-extract' ? 'wrong.bundle' : execute(file, args);
    await assert.rejects(
      prepareSubjectApps(
        invalidOutput,
        { revision: 'fixture', dirtyDiffSha256: null },
        built,
        inconsistentIdentity,
      ),
      /identity mismatch/,
    );
    await assert.rejects(readFile(join(invalidOutput, 'focus-app-identity.json')), /ENOENT/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
