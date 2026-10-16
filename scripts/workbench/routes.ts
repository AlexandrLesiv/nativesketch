import type { IncomingMessage, ServerResponse } from 'node:http';
import type { Jobs } from './jobs.ts';
import { SnapshotUnavailable } from './state.ts';
import type { SnapshotState } from './state.ts';
import type { Previews } from './preview.ts';
import { authorize, body } from './security.ts';
import { boundedRead, safeText } from './files.ts';
export interface RouteContext {
  root: string;
  assets: string;
  authority: string;
  capability: string;
  jobs: Jobs;
  snapshots: SnapshotState;
  previews: Previews;
}
export async function route(
  request: IncomingMessage,
  response: ServerResponse,
  context: RouteContext,
) {
  response.setHeader('X-Content-Type-Options', 'nosniff');
  response.setHeader('Referrer-Policy', 'no-referrer');
  response.setHeader('Cache-Control', 'no-store');
  response.setHeader('X-Frame-Options', 'DENY');
  response.setHeader(
    'Content-Security-Policy',
    "default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; frame-src http://127.0.0.1:*; object-src 'none'; base-uri 'none'; frame-ancestors 'none'",
  );
  const send = (value: unknown, status = 200) => {
    response.writeHead(status, { 'Content-Type': 'application/json' }).end(JSON.stringify(value));
  };
  try {
    const url = new URL(request.url ?? '/', `http://${context.authority}`);
    if (url.search) throw new Error('Query parameters are not supported');
    const path = url.pathname;
    if (!path.startsWith('/api/')) {
      if (
        request.headers.host !== context.authority ||
        request.method !== 'GET' ||
        (request.headers.origin && request.headers.origin !== `http://${context.authority}`) ||
        ['cross-site', 'same-site'].includes(String(request.headers['sec-fetch-site']))
      )
        throw new Error('Static request rejected');
      const files: Record<string, [string, string]> = {
        '/': ['index.html', 'text/html'],
        '/index.html': ['index.html', 'text/html'],
        '/assets/workbench.js': ['assets/workbench.js', 'text/javascript'],
        '/assets/workbench.css': ['assets/workbench.css', 'text/css'],
      };
      const selected = files[path];
      if (!selected) {
        send({ error: { code: 'not-found', message: 'Not found' } }, 404);
        return;
      }
      response
        .writeHead(200, { 'Content-Type': selected[1] })
        .end(await boundedRead(context.assets, selected[0], 4 * 1024 * 1024));
      return;
    }
    authorize(request, context.authority, context.capability, path === '/api/session');
    if (request.method === 'GET' && path === '/api/session') {
      send({
        capability: context.capability,
        policy: { foregroundExecution: false, maximumQueuedJobs: 4, maximumRetainedJobs: 30 },
      });
      return;
    }
    if (request.method === 'GET' && path === '/api/snapshot') {
      send(context.snapshots.current());
      return;
    }
    if (request.method === 'POST' && path === '/api/refresh') {
      await body(request, []);
      send(await context.snapshots.refresh());
      return;
    }
    if (request.method === 'GET' && path === '/api/jobs') {
      send({ jobs: context.jobs.list() });
      return;
    }
    if (request.method === 'POST' && path === '/api/jobs') {
      const data = await body(request, ['actionId']);
      send({ job: await context.jobs.enqueue(String(data.actionId)) }, 202);
      return;
    }
    const cancel = path.match(/^\/api\/jobs\/([a-f0-9-]{36})\/cancel$/);
    if (request.method === 'POST' && cancel) {
      await body(request, []);
      send({ job: context.jobs.cancel(cancel[1]) });
      return;
    }
    if (request.method === 'GET' && path === '/api/preview') {
      send({
        preview: context.previews.current,
        reason: context.previews.current ? null : 'No successful actual Wasm preview build yet.',
      });
      return;
    }
    if (request.method === 'GET' && path.startsWith('/api/artifacts/')) {
      const id = decodeURIComponent(path.slice('/api/artifacts/'.length));
      const indexed = new Set(context.jobs.list().flatMap((job) => job.artifactIds));
      try {
        const snapshot = context.snapshots.current().snapshot;
        for (const file of snapshot.coverage.data.evidencePaths) indexed.add(file);
        for (const row of snapshot.coverage.data.rows)
          for (const attempt of row.attempts)
            for (const file of attempt.artifacts)
              indexed.add(
                file.startsWith('artifacts/evidence/') ? file : `artifacts/evidence/${file}`,
              );
      } catch {
        /* Initial snapshot may be unavailable; job artifacts remain accessible. */
      }
      if (
        !indexed.has(id) ||
        !/^artifacts\/(evidence|workbench\/jobs)\/.+\.(json|log|txt|html)$/.test(id)
      )
        throw new Error('Artifact not indexed');
      const download = id.endsWith('/report.html');
      const bytes = await boundedRead(
        context.root,
        id,
        download ? 32 * 1024 * 1024 : 2 * 1024 * 1024,
      );
      response
        .writeHead(200, {
          'Content-Type': download ? 'application/octet-stream' : 'text/plain; charset=utf-8',
          'Content-Disposition': download
            ? 'attachment; filename="workbench-report.html"'
            : 'attachment; filename="evidence.txt"',
        })
        .end(download ? bytes : safeText(bytes.toString()));
      return;
    }
    send({ error: { code: 'not-found', message: 'Unsupported endpoint or method' } }, 404);
  } catch (error) {
    if (error instanceof SnapshotUnavailable && !response.headersSent) {
      send({ error: { code: 'snapshot-unavailable', message: error.message } }, 503);
      return;
    }
    if (!response.headersSent)
      send(
        {
          error: {
            code: 'request-rejected',
            message:
              'Request rejected: invalid authorization, input, unavailable data or service policy.',
          },
        },
        403,
      );
    else response.end();
  }
}
