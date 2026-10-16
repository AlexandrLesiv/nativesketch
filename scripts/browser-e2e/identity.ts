import { readFile, readdir, realpath } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { join } from 'node:path';
import { fileHash, sourceIdentity } from '../evidence/identity.ts';
import { root } from '../lib/context.ts';
export async function treeHash(directory: string) {
  const hash = createHash('sha256');
  async function visit(relative: string) {
    const entries = (await readdir(join(directory, relative), { withFileTypes: true })).sort(
      (a, b) => a.name.localeCompare(b.name),
    );
    for (const entry of entries) {
      const name = join(relative, entry.name);
      if (entry.isDirectory()) await visit(name);
      else if (entry.isFile()) {
        const bytes = await readFile(join(directory, name));
        hash.update(JSON.stringify([name, bytes.length])).update(bytes);
      } else throw new Error(`Unsupported identity entry: ${name}`);
    }
  }
  await visit('');
  return hash.digest('hex');
}
export async function browserIdentity(executable: string) {
  const browser = await realpath(executable);
  const testBinarySha256 = createHash('sha256')
    .update(await treeHash(join(root, 'apps/web/e2e')))
    .update(await treeHash(join(root, 'scripts/browser-e2e')))
    .update(await fileHash(join(root, 'scripts/test-web-e2e.ts')))
    .digest('hex');
  return {
    source: await sourceIdentity(root),
    binarySha256: await treeHash(join(root, 'artifacts/web')),
    testBinarySha256,
    browser,
    browserSha256: await fileHash(browser),
    lockSha256: await fileHash(join(root, 'package-lock.json')),
  };
}
