import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import vm from 'node:vm';
import { root } from '../lib/context.mjs';
await test('production assets resolve under Pages subpath and are all precached', async () => {
  const directory = join(root, 'artifacts/web');
  const worker = await readFile(join(directory, 'sw.js'), 'utf8');
  /** @type {Record<string,Function>} */
  const listeners = {};
  /** @type {string[]} */
  let assets = [];
  let cacheName = '';
  vm.runInNewContext(worker, {
    self: {
      addEventListener: (/** @type {string} */ name, /** @type {Function} */ callback) => {
        listeners[name] = callback;
      },
    },
    caches: {
      open: async (/** @type {string} */ name) => {
        cacheName = name;
        return {
          addAll: async (/** @type {string[]} */ paths) => {
            assets = paths;
          },
        };
      },
    },
  });
  /** @type {Promise<unknown> | undefined} */
  let installed;
  listeners.install({
    waitUntil: (/** @type {Promise<unknown>} */ promise) => {
      installed = promise;
    },
  });
  await installed;
  const base = new URL('https://example.test/nativesketch/');
  for (const asset of assets) {
    const url = new URL(asset, base);
    assert.ok(url.pathname.startsWith('/nativesketch/'));
    await readFile(join(directory, asset === './' ? 'index.html' : asset));
  }
  for (const file of ['main.js', 'engine.js']) {
    const source = await readFile(join(directory, file), 'utf8');
    for (const match of source.matchAll(/(?:from|import)\s*["']([^"']+)["']/g)) {
      assert.ok(
        assets.includes(`./${match[1].replace(/^\.\//, '')}`),
        `Uncached module ${match[1]}`,
      );
    }
  }
  const html = await readFile(join(directory, 'index.html'), 'utf8');
  assert.match(html, /src="main.js"/);
  assert.match(html, /href="style.css"/);
  const hash = createHash('sha256');
  for (const file of ['index.html', 'style.css', 'main.js', 'engine.js', 'sketch_wasm.wasm'])
    hash.update(await readFile(join(directory, file)));
  assert.equal(cacheName, `native-sketch-${hash.digest('hex').slice(0, 16)}`);
});
