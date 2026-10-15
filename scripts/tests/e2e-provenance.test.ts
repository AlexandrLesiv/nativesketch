import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, writeFile, rm, realpath, symlink } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { validateAttachments } from '../e2e/loaded-provenance.ts';

await test('each passing native test needs matching loaded-bundle evidence, including repeated launches', async () => {
  const directory = await realpath(await mkdtemp(join(tmpdir(), 'native-provenance-')));
  const id = 'xctest:NativeSketchUITests:Example/testJourney';
  const expected = {
    testBundle: '/attempt/Current.xctest',
    testBinary: '/attempt/Current.xctest/Contents/MacOS/Test',
    testBinarySha256: 'a'.repeat(64),
    testBundleSha256: 'b'.repeat(64),
  };
  const actual = {
    schemaVersion: 1,
    bundlePath: expected.testBundle,
    executablePath: expected.testBinary,
    binarySha256: expected.testBinarySha256,
  };
  const attachment = {
    suggestedHumanReadableName: 'loaded-test-bundle-provenance_0.txt',
    exportedFileName: 'a.txt',
  };
  const row = { testIdentifier: 'Example/testJourney()', attachments: [attachment] };
  try {
    await writeFile(join(directory, 'a.txt'), JSON.stringify(actual));
    await validateAttachments(directory, [row], [id], expected);
    await assert.rejects(validateAttachments(directory, [], [id], expected), /Missing/);
    await assert.rejects(validateAttachments(directory, [row, row], [id], expected), /ambiguous/);
    await assert.rejects(
      validateAttachments(directory, [{ ...row, attachments: [] }], [id], expected),
      /Missing/,
    );
    await writeFile(
      join(directory, 'b.txt'),
      JSON.stringify({ ...actual, bundlePath: '/stale/Embedded.xctest' }),
    );
    await assert.rejects(
      validateAttachments(
        directory,
        [{ ...row, attachments: [attachment, { ...attachment, exportedFileName: 'b.txt' }] }],
        [id],
        expected,
      ),
      /mismatch/,
    );
    await assert.rejects(
      validateAttachments(
        directory,
        [{ ...row, attachments: [{ ...attachment, exportedFileName: '../a.txt' }] }],
        [id],
        expected,
      ),
      /Invalid/,
    );
    await symlink('/etc/hosts', join(directory, 'escape.txt'));
    await assert.rejects(
      validateAttachments(
        directory,
        [{ ...row, attachments: [{ ...attachment, exportedFileName: 'escape.txt' }] }],
        [id],
        expected,
      ),
      /Unsafe/,
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
