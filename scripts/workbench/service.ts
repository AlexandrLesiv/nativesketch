import { createServer } from 'node:http';
import { lstat, open, unlink } from 'node:fs/promises';
import { join } from 'node:path';
import { randomBytes } from 'node:crypto';
import { Jobs } from './jobs.ts';
import { Previews } from './preview.ts';
import { SnapshotState } from './state.ts';
import { storageDirectory } from './storage.ts';
import { route } from './routes.ts';
export async function startWorkbench(options: { root: string; assets: string; port?: number }) {
  const { root, assets } = options;
  const storage = await storageDirectory(root, 'artifacts/workbench');
  await storageDirectory(root, 'artifacts/workbench/previews');
  const lockPath = join(storage, 'service.lock');
  const lock = await open(lockPath, 'wx');
  await lock.writeFile(
    JSON.stringify({ pid: process.pid, ownership: randomBytes(24).toString('hex') }),
  );
  const lockIdentity = await lock.stat();
  const snapshots = new SnapshotState(root);
  const previews = new Previews();
  const jobs = new Jobs(root, async (job, signal) => {
    if (job.actionId === 'preview-web') await previews.publish(root, job.id, job.source, signal);
  });
  const capability = randomBytes(32).toString('hex');
  let authority = '';
  const server = createServer((request, response) => {
    void route(request, response, {
      root,
      assets,
      authority,
      capability,
      jobs,
      snapshots,
      previews,
    });
  });
  server.requestTimeout = 15_000;
  server.headersTimeout = 10_000;
  server.maxHeadersCount = 32;
  let closing: Promise<void> | null = null;
  const close = () =>
    (closing ??= (async () => {
      server.closeAllConnections();
      await new Promise<void>((resolve) => server.close(() => resolve()));
      await jobs.close();
      await snapshots.close();
      await previews.close();
      await lock.close();
      const current = await lstat(lockPath).catch(() => null);
      if (current?.ino === lockIdentity.ino && current.dev === lockIdentity.dev)
        await unlink(lockPath);
    })());
  try {
    await new Promise<void>((resolve, reject) => {
      server.once('error', reject);
      server.listen(options.port ?? 0, '127.0.0.1', resolve);
    });
    const address = server.address();
    if (!address || typeof address === 'string') throw new Error('Loopback binding failed');
    authority = `127.0.0.1:${address.port}`;
    try {
      await snapshots.refresh();
    } catch {
      /* Service exposes explicit snapshot-unavailable state. */
    }
    return { url: `http://${authority}/`, close };
  } catch (error) {
    await close();
    throw error;
  }
}
