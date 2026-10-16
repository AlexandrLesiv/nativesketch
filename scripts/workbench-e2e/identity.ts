import type { JSONReportSpec } from 'playwright/types/testReporter';
import { basename, join } from 'node:path';
import { createHash } from 'node:crypto';
import { realpath as resolveRealpath } from 'node:fs/promises';
import { treeHash } from '../browser-e2e/identity.ts';
import { sourceIdentity, fileHash } from '../evidence/identity.ts';
import { root } from '../lib/context.ts';
export function testId(spec: JSONReportSpec) {
  const id = /^DX-[A-Z]+-\d{3}\b/.exec(spec.title)?.[0];
  if (!id) throw new Error('Workbench test needs a stable DX acceptance ID');
  return `playwright:workbench:${basename(spec.file)}:${id}`;
}
export async function identity(executable: string) {
  const browser = await resolveRealpath(executable);
  return {
    source: await sourceIdentity(root),
    binarySha256: await treeHash(join(root, 'apps/workbench/dist')),
    testBinarySha256: createHash('sha256')
      .update(await treeHash(join(root, 'apps/workbench/e2e')))
      .update(await treeHash(join(root, 'scripts/workbench-e2e')))
      .update(await treeHash(join(root, 'scripts/workbench')))
      .update(await fileHash(join(root, 'scripts/test-workbench.ts')))
      .digest('hex'),
    browserSha256: await fileHash(browser),
    lockSha256: await fileHash(join(root, 'package-lock.json')),
  };
}
