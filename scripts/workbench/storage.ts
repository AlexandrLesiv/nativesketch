import { lstat, mkdir, realpath } from 'node:fs/promises';
import { join } from 'node:path';
/** Create only project-local storage, rejecting pre-existing symlink components. */
export async function storageDirectory(root: string, identifier: string) {
  if (!/^artifacts\/workbench(?:\/[a-zA-Z0-9._-]+)*$/.test(identifier))
    throw new Error('Invalid workbench storage path');
  let directory = await realpath(root);
  for (const part of identifier.split('/')) {
    directory = join(directory, part);
    try {
      await mkdir(directory);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'EEXIST') throw error;
    }
    const entry = await lstat(directory);
    if (!entry.isDirectory() || entry.isSymbolicLink()) throw new Error('Unsafe storage component');
  }
  return directory;
}
