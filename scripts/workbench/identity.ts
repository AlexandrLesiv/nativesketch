import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { createHash } from 'node:crypto';
import { boundedRead } from './files.ts';
/** Same revision/diff algorithm as evidence identity, with bounded confined untracked reads. */
export async function sourceIdentity(root: string) {
  const git = async (args: string[]) =>
    (await promisify(execFile)('git', args, { cwd: root, maxBuffer: 4_000_000, timeout: 30_000 }))
      .stdout;
  const revision = (await git(['rev-parse', 'HEAD'])).trim();
  const diff = await git(['diff', 'HEAD', '--binary']);
  const untracked = (await git(['ls-files', '--others', '--exclude-standard', '-z']))
    .split('\0')
    .filter(Boolean)
    .sort();
  if (untracked.length > 10000) throw new Error('Identity file count exceeds bounds');
  if (!diff && !untracked.length) return { revision, dirtyDiffSha256: null };
  const digest = createHash('sha256').update(diff);
  let total = 0;
  for (const name of untracked) {
    const bytes = await boundedRead(root, name);
    total += bytes.length;
    if (total > 12 * 1024 * 1024) throw new Error('Identity byte bound exceeded');
    digest.update(`\0${Buffer.byteLength(name)}:${name}:${bytes.length}:`).update(bytes);
  }
  return { revision, dirtyDiffSha256: digest.digest('hex') };
}
