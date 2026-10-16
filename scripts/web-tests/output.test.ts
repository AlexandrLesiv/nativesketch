import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { root } from '../lib/context.ts';
import { workerHarness } from './worker-harness.ts';
import { webAssetManifest } from '../../apps/web/asset-manifest.ts';

await test('every emitted dependency is hashed and precached beneath the Pages subpath', async () => {
  const directory = join(root, 'artifacts/web');
  const worker = workerHarness(await readFile(join(directory, 'sw.js'), 'utf8'));
  await worker.install();
  const manifest = await webAssetManifest(directory);
  assert.deepEqual(worker.state.assets, manifest.assets);
  const base = new URL('https://example.test/nativesketch/');
  for (const asset of worker.state.assets) {
    const url = new URL(asset, base);
    assert.ok(url.pathname.startsWith('/nativesketch/'));
    await readFile(join(directory, asset === './' ? 'index.html' : asset));
  }
  const modules = manifest.files.filter((file) => file.endsWith('.js'));
  assert.ok(
    modules.length > 2,
    'extraction must emit transitive modules, not only two old entries',
  );
  for (const file of modules) {
    const source = await readFile(join(directory, file), 'utf8');
    for (const match of source.matchAll(/(?:from|import)\s*["']([^"']+)["']/g)) {
      const resolved = new URL(match[1], new URL(file, base));
      assert.equal(resolved.origin, base.origin);
      assert.ok(resolved.pathname.startsWith(base.pathname));
      assert.ok(
        worker.state.assets.includes(`./${resolved.pathname.slice(base.pathname.length)}`),
        `Uncached module ${match[1]} from ${file}`,
      );
    }
  }
  const html = await readFile(join(directory, 'index.html'), 'utf8');
  assert.match(html, /src="main.js"/);
  assert.match(html, /href="style.css"/);
  const hash = createHash('sha256');
  for (const file of manifest.files) {
    const bytes = await readFile(join(directory, file));
    hash.update(JSON.stringify([file, bytes.length])).update(bytes);
  }
  assert.equal(worker.state.cacheName, `native-sketch-${hash.digest('hex').slice(0, 16)}`);
});

await test('failed cache population rejects installation without deleting the previous cache', async () => {
  const worker = workerHarness(
    await readFile(join(root, 'artifacts/web/sw.js'), 'utf8'),
    new Error('missing emitted module'),
  );
  await assert.rejects(worker.install(), /missing emitted module/);
  assert.deepEqual(worker.state.deleted, []);
});
