import { readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { join } from 'node:path';
import { fileHash } from '../evidence/identity.ts';
export async function classIdentity(root: string) {
  const files: { path: string; sha256: string }[] = [];
  async function collect(directory: string): Promise<void> {
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) await collect(path);
      else if (path.endsWith('.class'))
        files.push({ path: path.slice(root.length + 1), sha256: await fileHash(path) });
    }
  }
  await collect(join(root, 'apps/android/app/build/intermediates/built_in_kotlinc'));
  files.sort((a, b) => a.path.localeCompare(b.path));
  if (!files.length) throw new Error('No compiled JVM classes');
  return { files, sha256: createHash('sha256').update(JSON.stringify(files)).digest('hex') };
}
