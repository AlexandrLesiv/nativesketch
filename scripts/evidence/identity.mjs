import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
/** @param {string} path */
export async function fileHash(path) {
  return createHash('sha256')
    .update(await readFile(path))
    .digest('hex');
}
/** Includes tracked changes and every nonignored untracked file, with names and lengths. @param {string} root */
export async function sourceIdentity(root) {
  const git = async (/** @type {string[]} */ args) =>
    (await promisify(execFile)('git', args, { cwd: root, maxBuffer: 4000000, timeout: 30000 }))
      .stdout;
  const revision = (await git(['rev-parse', 'HEAD'])).trim();
  const diff = await git(['diff', 'HEAD', '--binary']);
  const untracked = (await git(['ls-files', '--others', '--exclude-standard', '-z']))
    .split('\0')
    .filter(Boolean)
    .sort();
  if (!diff && !untracked.length) return { revision, dirtyDiffSha256: null };
  const digest = createHash('sha256').update(diff);
  for (const name of untracked) {
    const bytes = await readFile(resolve(root, name));
    digest.update(`\0${Buffer.byteLength(name)}:${name}:${bytes.length}:`).update(bytes);
  }
  return { revision, dirtyDiffSha256: digest.digest('hex') };
}
