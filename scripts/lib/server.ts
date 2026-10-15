import { isContained } from './paths.ts';
import { createServer } from 'node:http';
import { readFile, realpath } from 'node:fs/promises';
import { resolve, extname } from 'node:path';
/** Loopback-only static production server, with traversal/symlink containment. */

export async function serve(directory: string, port: number, signal: AbortSignal) {
  const base = await realpath(directory);

  const types: Record<string, string> = {
    '.html': 'text/html',
    '.js': 'text/javascript',
    '.css': 'text/css',
    '.wasm': 'application/wasm',
  };
  const server = createServer(async (request, response) => {
    try {
      if (!['GET', 'HEAD'].includes(request.method ?? '')) {
        response.writeHead(405).end();
        return;
      }
      const pathname = decodeURIComponent(new URL(request.url ?? '/', 'http://localhost').pathname);
      const path = await realpath(
        resolve(base, `.${pathname.endsWith('/') ? `${pathname}index.html` : pathname}`),
      );
      if (!isContained(base, path)) {
        response.writeHead(403).end();
        return;
      }
      const bytes = await readFile(path);
      response.writeHead(200, {
        'Content-Type': types[extname(path)] ?? 'application/octet-stream',
      });
      response.end(request.method === 'HEAD' ? undefined : bytes);
    } catch {
      response.writeHead(404).end();
    }
  });
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(port, '127.0.0.1', () => resolve(undefined));
  });
  console.log(`Serving ${base} at http://127.0.0.1:${port}`);
  signal.addEventListener(
    'abort',
    () => {
      server.closeAllConnections();
      server.close();
    },
    { once: true },
  );
  return server;
}
