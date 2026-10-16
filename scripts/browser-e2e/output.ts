import { lstat, realpath, mkdir } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { isContained } from '../lib/paths.ts';
/** Refuse symlink escapes before creating any directories or replacing evidence. */
export async function prepareOutput(root: string, output: string) {
  const artifacts = resolve(root, 'artifacts');
  if (!isContained(artifacts, output) || output === artifacts) throw new Error('Unsafe output');
  let ancestor = dirname(output);
  while (true) {
    try {
      await lstat(ancestor);
      break;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
      ancestor = dirname(ancestor);
    }
  }
  const resolvedRoot = await realpath(root);
  const resolvedAncestor = await realpath(ancestor);
  if (!isContained(resolvedRoot, resolvedAncestor))
    throw new Error('Output ancestor escapes project');
  // Existing artifact parents may not alias another directory, even within the project.
  if (resolvedAncestor !== resolve(resolvedRoot, ancestor.slice(resolve(root).length + 1)))
    throw new Error('Output ancestor is a symlink alias');
  await mkdir(dirname(output), { recursive: true });
  await mkdir(output);
}
