import { createServer } from 'node:http';
import type { Server } from 'node:http';
import { mkdir, opendir, writeFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import type { Preview, Source } from './contracts/index.ts';
import { sourceIdentity } from './identity.ts';
import { boundedRead, hash } from './files.ts';
const mime: Record<string, string> = {
  html: 'text/html',
  js: 'text/javascript',
  css: 'text/css',
  wasm: 'application/wasm',
  json: 'application/json',
};
export class Previews {
  current: Preview | null = null;
  private servers: Server[] = [];
  private retainedBytes = 0;
  async publish(root: string, id: string, source: Source, signal: AbortSignal) {
    if (this.servers.length >= 8)
      throw new Error('Preview retention limit reached; restart workbench to release old previews');
    const destination = join(root, 'artifacts/workbench/previews', id);
    await mkdir(destination, { recursive: false });
    const files: string[] = [];
    const contents = new Map<string, Buffer>();
    let entriesCount = 0;
    let total = 0;
    const hashes: string[] = [];
    const copy = async (directory = '') => {
      if (directory.split('/').length > 16) throw new Error('Preview directory depth bound');
      const entries = await opendir(join(root, 'artifacts/web', directory));
      for await (const entry of entries) {
        if (++entriesCount > 2000) throw new Error('Preview entry count bound');
        const name = directory ? `${directory}/${entry.name}` : entry.name;
        if (entry.isSymbolicLink()) throw new Error('Preview symlink rejected');
        if (entry.isDirectory()) {
          await mkdir(join(destination, name));
          await copy(name);
        } else {
          const bytes = await boundedRead(root, `artifacts/web/${name}`, 16 * 1024 * 1024);
          total += bytes.length;
          files.push(name);
          hashes.push(`${name}:${hash(bytes)}`);
          if (
            total > 64 * 1024 * 1024 ||
            total + this.retainedBytes > 128 * 1024 * 1024 ||
            files.length > 1000
          )
            throw new Error('Preview exceeds bounds');
          contents.set(name, Buffer.from(bytes));
          await writeFile(join(destination, name), bytes, { flag: 'wx' });
        }
      }
    };
    try {
      await copy();
      if (!files.includes('index.html') || !files.some((name) => name.endsWith('.wasm')))
        throw new Error('Actual Wasm preview output missing');
      signal.throwIfAborted();
      let authority = '';
      const server = createServer(async (request, response) => {
        try {
          if (request.headers.host !== authority || request.method !== 'GET')
            throw new Error('Invalid preview request');
          const name =
            decodeURIComponent(
              new URL(request.url ?? '/', `http://${authority}`).pathname.slice(1),
            ) || 'index.html';
          if (!files.includes(name)) {
            response.writeHead(404).end();
            return;
          }
          const body = contents.get(name);
          response
            .writeHead(200, {
              'Content-Type': mime[name.split('.').at(-1) ?? ''] ?? 'application/octet-stream',
              'X-Content-Type-Options': 'nosniff',
              'Referrer-Policy': 'no-referrer',
            })
            .end(body);
        } catch {
          response.writeHead(403).end('Preview request rejected');
        }
      });
      await new Promise<void>((resolve, reject) => {
        server.once('error', reject);
        server.listen(0, '127.0.0.1', resolve);
      });
      const address = server.address();
      if (!address || typeof address === 'string') throw new Error('Preview bind failed');
      authority = `127.0.0.1:${address.port}`;
      try {
        signal.throwIfAborted();
        if (JSON.stringify(source) !== JSON.stringify(await sourceIdentity(root)))
          throw new Error('Source changed before preview publication');
        signal.throwIfAborted();
      } catch (error) {
        server.closeAllConnections();
        await new Promise<void>((resolve) => server.close(() => resolve()));
        throw error;
      }
      this.servers.push(server);
      this.retainedBytes += total;
      this.current = {
        source,
        sha256: hash(hashes.sort().join('\n')),
        url: `http://${authority}/`,
        builtAt: new Date().toISOString(),
        renderer: 'Canvas2D',
      };
    } catch (error) {
      await rm(destination, { recursive: true, force: true });
      throw error;
    }
  }
  async close() {
    await Promise.all(
      this.servers.map(
        (server) =>
          new Promise<void>((resolve) => {
            server.closeAllConnections();
            server.close(() => resolve());
          }),
      ),
    );
  }
}
