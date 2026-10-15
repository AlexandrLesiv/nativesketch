import { mkdir, stat } from 'node:fs/promises';
import { join } from 'node:path';
import { main, command, root } from './lib/context.mjs';
await main(async () => {
  const [name, ...extra] = process.argv.slice(2);
  if (!name || extra.length || !/^[a-z0-9][a-z0-9-]{0,59}$/.test(name))
    throw new Error('Pass one lowercase task name (up to 60 letters, numbers, hyphens)');
  if (!(await stat(join(root, '.git'))).isDirectory())
    throw new Error('Run from the primary repository, not an existing worktree');
  await command('git', ['rev-parse', '--verify', 'HEAD'], { capture: true });
  if (await command('git', ['status', '--porcelain'], { capture: true }))
    throw new Error('Commit intended base changes first; worktrees start from HEAD');
  await mkdir(join(root, '.worktrees'), { recursive: true });
  const path = join(root, '.worktrees', name);
  await command('git', ['worktree', 'add', '-b', `codex/${name}`, path, 'HEAD']);
  console.log(`Give the worker this checkout: ${path}`);
});
