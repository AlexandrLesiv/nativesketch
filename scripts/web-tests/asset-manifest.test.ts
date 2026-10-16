import test from 'node:test';
import assert from 'node:assert/strict';
import { cp, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { root } from '../lib/context.ts';
import { webAssetManifest } from '../../apps/web/asset-manifest.ts';

await test('missing transitive output fails validation and changing it changes the cache version', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'native-sketch-web-assets-'));
  try {
    await cp(join(root, 'artifacts/web'), directory, { recursive: true });
    const before = await webAssetManifest(directory);
    const leaf = before.files.find((file) => file.endsWith('elements.js'));
    assert.ok(leaf, 'the extracted DOM helper must be an emitted module');
    const path = join(directory, leaf);
    const bytes = await readFile(path);
    await writeFile(path, Buffer.concat([bytes, Buffer.from('\n// cache invalidation fixture\n')]));
    const changed = await webAssetManifest(directory);
    assert.notEqual(changed.version, before.version);
    await rm(path);
    await assert.rejects(webAssetManifest(directory), /inventory does not match/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

await test('unknown transitive imports cannot be omitted from the precache graph', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'native-sketch-web-graph-'));
  try {
    await cp(join(root, 'artifacts/web'), directory, { recursive: true });
    const path = join(directory, 'module-inventory.json');
    const inventory = JSON.parse(await readFile(path, 'utf8')) as {
      files: string[];
      modules: { file: string; imports: string[] }[];
    };
    inventory.modules[0].imports.push('missing-dependency.js');
    await writeFile(path, JSON.stringify(inventory));
    await assert.rejects(webAssetManifest(directory), /Unlisted module dependency/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
