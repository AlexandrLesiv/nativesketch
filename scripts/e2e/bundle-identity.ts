import { createHash } from 'node:crypto';
import { lstat, readdir, readFile, readlink } from 'node:fs/promises';
import { join } from 'node:path';
import { promisify } from 'node:util';
import { execFile } from 'node:child_process';
import { fileHash } from '../evidence/identity.ts';

/** Hash paths, modes, symlink targets and bytes without following bundle symlinks. */
export async function bundleHash(root: string) {
  if (!(await lstat(root)).isDirectory()) throw new Error('Bundle root must be a real directory');
  const hash = createHash('sha256');
  async function visit(relative: string) {
    const path = join(root, relative);
    const info = await lstat(path);
    const bytes = info.isSymbolicLink()
      ? Buffer.from(await readlink(path))
      : info.isFile()
        ? await readFile(path)
        : Buffer.alloc(0);
    if (!info.isDirectory() && !info.isFile() && !info.isSymbolicLink())
      throw new Error(`Unsupported bundle entry: ${relative}`);
    hash.update(JSON.stringify([relative, info.mode, bytes.length])).update(bytes);
    if (info.isDirectory())
      for (const name of (await readdir(path)).sort()) await visit(join(relative, name));
  }
  await visit('');
  return hash.digest('hex');
}

export async function inspectRunner(path: string) {
  const execute = (file: string, args: string[]) =>
    promisify(execFile)(file, args, { timeout: 30_000, maxBuffer: 1_000_000 });
  await execute('/usr/bin/codesign', ['--verify', '--strict', '--deep', path]);
  const signature = await execute('/usr/bin/codesign', ['-d', '-r-', path]);
  const requirement = (signature.stdout + signature.stderr)
    .split('\n')
    .find((line) => line.includes('designated =>'));
  if (!requirement) throw new Error('Runner designated requirement missing');
  const identifier = (
    await execute('/usr/bin/plutil', [
      '-extract',
      'CFBundleIdentifier',
      'raw',
      '-o',
      '-',
      join(path, 'Contents/Info.plist'),
    ])
  ).stdout.trim();
  const executable = (
    await execute('/usr/bin/plutil', [
      '-extract',
      'CFBundleExecutable',
      'raw',
      '-o',
      '-',
      join(path, 'Contents/Info.plist'),
    ])
  ).stdout.trim();
  if (!executable || executable.includes('/') || executable === '..')
    throw new Error('Invalid runner executable name');
  return {
    identifier,
    requirement,
    executableSha256: await fileHash(join(path, 'Contents/MacOS', executable)),
    treeSha256: await bundleHash(path),
  };
}
