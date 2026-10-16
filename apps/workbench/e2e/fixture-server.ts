import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import type { Job } from '../../../scripts/workbench/contracts/index.ts';
import { snapshotFixture } from './fixture.ts';
/** Disposable protocol fixture; it never starts a process or substitutes for service tests. */
export async function fixtureServer(): Promise<{ url: string; close: () => Promise<void> }> {
  const snapshot = snapshotFixture();
  const jobs: Job[] = [];
  const assets = resolve(import.meta.dirname, '../dist');
  const server = createServer((req, res) => {
    void (async () => {
      res.setHeader('Content-Type', 'application/json');
      const path = req.url ?? '/';
      if (path === '/api/session') {
        res.end(
          JSON.stringify({
            capability: 'fixture-capability',
            policy: { foregroundExecution: false, maximumQueuedJobs: 4, maximumRetainedJobs: 30 },
          }),
        );
        return;
      }
      if (
        path.startsWith('/api/') &&
        req.headers['x-workbench-capability'] !== 'fixture-capability'
      ) {
        res.statusCode = 403;
        res.end('{}');
        return;
      }
      if (path === '/api/snapshot' || path === '/api/refresh')
        res.end(JSON.stringify({ snapshot, stale: true, diagnostics: [] }));
      else if (path === '/api/preview')
        res.end(JSON.stringify({ preview: null, reason: 'Fixture has no built preview' }));
      else if (path === '/api/jobs' && req.method === 'GET') {
        for (const job of jobs)
          if (job.state === 'running') {
            job.log += '\nFixture heartbeat';
            job.outputBytes += 18;
          }
        res.end(JSON.stringify({ jobs }));
      } else if (path === '/api/jobs' && req.method === 'POST') {
        const job: Job = {
          id: `fixture-${jobs.length}`,
          actionId: 'fixture-check',
          state: jobs.length ? 'running' : 'failed',
          source: snapshot.source,
          queuedAt: snapshot.generatedAt,
          startedAt: snapshot.generatedAt,
          endedAt: jobs.length ? null : snapshot.generatedAt,
          commandPreview: 'fixture-check-only',
          exitCode: jobs.length ? null : 7,
          signal: null,
          reason: jobs.length ? null : 'Deliberate fixture failure',
          log: '<script>unsafe()</script>',
          outputBytes: 27,
          sourceChanged: true,
          artifactIds: ['fixture.txt'],
        };
        jobs.push(job);
        res.end(JSON.stringify({ job }));
      } else if (path === '/api/artifacts/fixture.txt') {
        res.setHeader('Content-Type', 'text/plain');
        res.end('Fixture evidence bytes');
      } else if (path.endsWith('/cancel')) {
        const job = jobs.at(-1);
        if (job) {
          job.state = 'cancelled';
          job.reason = 'Fixture cancellation complete';
        }
        res.end(JSON.stringify({ job }));
      } else {
        const files = new Map([
          ['/', ['index.html', 'text/html']],
          ['/assets/workbench.js', ['assets/workbench.js', 'application/javascript']],
          ['/assets/workbench.css', ['assets/workbench.css', 'text/css']],
        ]);
        const file = files.get(path);
        if (!file) {
          res.statusCode = 404;
          res.end('{}');
          return;
        }
        res.setHeader('Content-Type', file[1] ?? 'text/plain');
        res.end(await readFile(resolve(assets, file[0] ?? 'index.html')));
      }
    })().catch((error: unknown) => {
      res.statusCode = 500;
      res.end(String(error));
    });
  });
  await new Promise<void>((done) => server.listen(0, '127.0.0.1', done));
  const address = server.address();
  if (!address || typeof address === 'string') throw new Error('Fixture address missing');
  return {
    url: `http://127.0.0.1:${address.port}/`,
    close: () =>
      new Promise((done, reject) => {
        server.close((error) => (error ? reject(error) : done()));
        server.closeAllConnections();
      }),
  };
}
