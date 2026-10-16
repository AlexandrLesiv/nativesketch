import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { rendererIdentity, testClassification } from './renderer.ts';
await test('renderer evidence binds installed full bundle, declarations and emitted output mutations', async () => {
  const root = await mkdtemp(join(tmpdir(), 'skia-identity-'));
  const files = [
    'node_modules/canvaskit-wasm/bin/full/canvaskit.js',
    'node_modules/canvaskit-wasm/bin/full/canvaskit.wasm',
    'node_modules/canvaskit-wasm/types/index.d.ts',
    'apps/web/types/compat.d.ts',
    'artifacts/web/module-inventory.json',
  ];
  try {
    for (const file of files) {
      await mkdir(join(root, file, '..'), { recursive: true });
      await writeFile(join(root, file), 'original');
    }
    const packageFile = join(root, 'node_modules/canvaskit-wasm/package.json');
    await writeFile(packageFile, JSON.stringify({ version: '0.42.0' }));
    for (const file of files) {
      const before = await rendererIdentity(root);
      await writeFile(join(root, file), 'changed');
      assert.notDeepEqual(await rendererIdentity(root), before, file);
    }
    await writeFile(packageFile, JSON.stringify({ version: 'different' }));
    await assert.rejects(rendererIdentity(root), /version/);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
await test('web evidence distinguishes CPU raster, compiler and injected transport routes', () => {
  assert.equal(
    testClassification('node:apps/web/tests/skia-raster.test.ts:case').level,
    'rendering',
  );
  assert.match(
    testClassification('node:apps/web/tests/skia-precision.test.ts:case').inputRoute,
    /CPU raster/,
  );
  assert.equal(testClassification('node:apps/web/tests/skia-types.test.ts:case').level, 'tooling');
  assert.match(
    testClassification('node:apps/web/tests/injected-failures.test.ts:case').inputRoute,
    /injected/,
  );
  assert.equal(testClassification('node:apps/web/tests/editor.test.ts:case').level, 'bridge');
});
