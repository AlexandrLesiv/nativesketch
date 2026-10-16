import { readFile, realpath, stat, writeFile } from 'node:fs/promises';
import { basename, dirname, join } from 'node:path';
import { command } from '../lib/context.ts';
import { bundleHash, inspectRunner } from './bundle-identity.ts';
import { fileHash } from '../evidence/identity.ts';

type Expected = {
  testBundle: string;
  testBinary: string;
  testBinarySha256: string;
  testBundleSha256: string;
};
function record(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

export async function validateAttachments(
  directory: string,
  manifest: unknown,
  passed: string[],
  expected: Expected,
) {
  if (!Array.isArray(manifest)) throw new Error('Invalid XCTest attachment manifest');
  const canonical = await realpath(directory);
  for (const id of passed) {
    const identifier = id.split(':').at(-1);
    const rows = manifest.filter(
      (row: unknown) =>
        record(row) &&
        typeof row.testIdentifier === 'string' &&
        row.testIdentifier.replace(/\(\)$/, '') === identifier,
    );
    if (rows.length !== 1 || !record(rows[0]) || !Array.isArray(rows[0].attachments))
      throw new Error(`Missing or ambiguous loaded-bundle provenance for ${id}`);
    const attachments: unknown[] = rows[0].attachments;
    let matched = 0;
    for (const attachment of attachments) {
      if (
        !record(attachment) ||
        typeof attachment.suggestedHumanReadableName !== 'string' ||
        !attachment.suggestedHumanReadableName.startsWith('loaded-test-bundle-provenance')
      )
        continue;
      const name = attachment.exportedFileName;
      if (typeof name !== 'string' || basename(name) !== name)
        throw new Error('Invalid provenance attachment path');
      const path = await realpath(join(canonical, name));
      if (dirname(path) !== canonical || (await stat(path)).size > 16_384)
        throw new Error('Unsafe provenance attachment');
      const contents = await readFile(path, 'utf8');
      const actual: unknown = JSON.parse(contents);
      if (
        !record(actual) ||
        actual.schemaVersion !== 1 ||
        actual.bundlePath !== expected.testBundle ||
        actual.executablePath !== expected.testBinary ||
        actual.binarySha256 !== expected.testBinarySha256
      )
        throw new Error(`Loaded test bundle mismatch for ${id}`);
      matched++;
    }
    if (!matched) throw new Error(`Missing loaded-bundle provenance for ${id}`);
  }
}

/** Passing XCTest output alone cannot certify a possibly stale embedded test bundle. */
export async function validateRuntimeProvenance(
  output: string,
  passed: string[],
  expected: Expected & { host: string; hostIdentity: unknown },
) {
  try {
    const observed = await inspectRunner(expected.host);
    if (!record(expected.hostIdentity) || expected.hostIdentity.treeSha256 !== observed.treeSha256)
      throw new Error('Frozen XCTest host changed during execution');
    if (
      (await fileHash(expected.testBinary)) !== expected.testBinarySha256 ||
      (await bundleHash(expected.testBundle)) !== expected.testBundleSha256
    )
      throw new Error('External test bundle changed during execution');
    const directory = join(output, 'attachments');
    await command(
      'xcrun',
      [
        'xcresulttool',
        'export',
        'attachments',
        '--path',
        join(output, 'journey.xcresult'),
        '--output-path',
        directory,
      ],
      {
        capture: true,
        timeout: 60_000,
        env: {
          DEVELOPER_DIR: process.env.DEVELOPER_DIR ?? '/Applications/Xcode.app/Contents/Developer',
        },
      },
    );
    await validateAttachments(
      directory,
      JSON.parse(await readFile(join(directory, 'manifest.json'), 'utf8')),
      passed,
      expected,
    );
    await writeFile(
      join(output, 'provenance-validation.json'),
      JSON.stringify({ valid: true, passed }, null, 2),
    );
  } catch (error) {
    await writeFile(
      join(output, 'provenance-validation.json'),
      JSON.stringify({ valid: false, reason: String(error), passed }, null, 2),
    );
    throw error;
  }
}
