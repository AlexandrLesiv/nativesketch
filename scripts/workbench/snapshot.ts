import { tasks } from './adapters/tasks.ts';
import { run } from '../lib/process.ts';
import { sourceIdentity } from './identity.ts';
import type { ProjectSnapshot } from './contracts/index.ts';
import { sanitize } from './files.ts';
import { documents } from './adapters/documents.ts';
import { dependencies } from './adapters/dependencies.ts';
import { coverage } from './adapters/coverage.ts';
import { metrics } from './adapters/metrics.ts';
import { actionCatalog } from './actions.ts';
export async function collectSnapshot(root: string): Promise<ProjectSnapshot> {
  const before = await sourceIdentity(root);
  const git = (args: string[]) => run('git', args, { cwd: root, capture: true, timeout: 30_000 });
  const tracked = (await git(['ls-files', '-z'])).split('\0').filter(Boolean);
  const paths = [
    ...new Set([
      ...tracked,
      ...(await git(['ls-files', '-z', '--others', '--exclude-standard']))
        .split('\0')
        .filter(Boolean),
    ]),
  ];
  if (paths.length > 20000) throw new Error('Repository file count exceeds snapshot bound');
  const snapshot: ProjectSnapshot = {
    schemaVersion: 1,
    generatedAt: new Date().toISOString(),
    source: before,
    branch: await git(['branch', '--show-current']),
    host: { platform: process.platform, architecture: process.arch },
    tasks: await tasks(root),
    documents: await documents(root, tracked),
    dependencies: await dependencies(root),
    coverage: await coverage(root, tracked, before),
    metrics: await metrics(root, paths, before),
    actions: { state: 'ready', diagnostics: [], data: await actionCatalog() },
  };
  if (JSON.stringify(before) !== JSON.stringify(await sourceIdentity(root)))
    throw new Error('Source changed during refresh; previous snapshot remains stale');
  const safe = sanitize(snapshot);
  if (Buffer.byteLength(JSON.stringify(safe)) > 24 * 1024 * 1024)
    throw new Error('Snapshot exceeds 24 MiB');
  return safe;
}
