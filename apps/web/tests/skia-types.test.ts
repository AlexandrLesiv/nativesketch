import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';
await test('CanvasKit published API and built-in WebGPU remain checked with legacy globals deduplicated', async () => {
  const directory = await mkdtemp(resolve('artifacts/skia-types-'));
  try {
    await writeFile(
      join(directory, 'tsconfig.json'),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          noEmit: true,
          target: 'ES2022',
          module: 'ESNext',
          moduleResolution: 'Bundler',
          lib: ['ES2022', 'DOM'],
          types: [],
          typeRoots: [resolve('apps/web/types')],
        },
        files: ['fixture.ts'],
      }),
    );
    const compile = () =>
      spawnSync(
        process.execPath,
        ['node_modules/typescript/bin/tsc', '-p', join(directory, 'tsconfig.json')],
        { encoding: 'utf8' },
      );
    await writeFile(
      join(directory, 'fixture.ts'),
      `import type {CanvasKit} from 'canvaskit-wasm';
      declare const kit:CanvasKit; declare const device:GPUDevice;
      kit.Color4f(1,0,0,1); device.createBuffer({size:4,usage:4});`,
    );
    const valid = compile();
    assert.equal(valid.status, 0, valid.stdout + valid.stderr);
    await writeFile(
      join(directory, 'fixture.ts'),
      `import type {CanvasKit} from 'canvaskit-wasm';
      declare const kit:CanvasKit; declare const device:GPUDevice;
      kit.Color4f('invalid',0,0,1);
      device.createBuffer({size:'invalid',usage:4});`,
    );
    const invalid = compile();
    assert.notEqual(invalid.status, 0);
    assert.match(invalid.stdout, /TS2345/);
    assert.match(invalid.stdout, /TS2322/);
    assert.equal((invalid.stdout.match(/error TS/g) ?? []).length, 2, invalid.stdout);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
