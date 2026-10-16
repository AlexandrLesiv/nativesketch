import { mkdir, copyFile, readFile, writeFile, mkdtemp, rm } from 'node:fs/promises';
import { webAssetManifest } from '../../apps/web/asset-manifest.ts';
import { join } from 'node:path';
import { command, root } from '../lib/context.ts';
export async function buildWasm() {
  await command('cargo', [
    'build',
    '--locked',
    '--release',
    '-p',
    'sketch-wasm',
    '--target',
    'wasm32-unknown-unknown',
  ]);
  await mkdir(join(root, 'artifacts/wasm-legacy'), { recursive: true });
  await copyFile(
    join(root, 'target/wasm32-unknown-unknown/release/sketch_wasm.wasm'),
    join(root, 'artifacts/wasm-legacy/sketch_wasm.wasm'),
  );
}
export async function buildWeb() {
  await rm(join(root, 'artifacts/web'), { recursive: true, force: true });
  await command('cargo', [
    'build',
    '--locked',
    '--release',
    '-p',
    'sketch-app-wasm',
    '--target',
    'wasm32-unknown-unknown',
  ]);
  await mkdir(join(root, 'artifacts/web'), { recursive: true });
  await copyFile(
    join(root, 'target/wasm32-unknown-unknown/release/sketch_app_wasm.wasm'),
    join(root, 'artifacts/web/sketch_app_wasm.wasm'),
  );
  await command(process.execPath, [
    'node_modules/typescript/bin/tsc',
    '-p',
    'apps/web',
    '--noEmit',
  ]);
  await command(process.execPath, [
    'node_modules/vite/bin/vite.js',
    'build',
    '--config',
    'apps/web/vite.config.ts',
  ]);
  const output = join(root, 'artifacts/web');
  for (const file of ['index.html', 'style.css'])
    await copyFile(join(root, 'apps/web', file), join(output, file));
  await copyFile(
    join(root, 'node_modules/canvaskit-wasm/bin/full/canvaskit.wasm'),
    join(output, 'canvaskit.wasm'),
  );
  await copyFile(
    join(root, 'node_modules/canvaskit-wasm/LICENSE'),
    join(output, 'CANVASKIT-LICENSE.txt'),
  );
  const manifest = await webAssetManifest(output);
  const worker = (await readFile(join(root, 'apps/web/sw.js'), 'utf8'))
    .replace('__CACHE_VERSION__', manifest.version)
    .replace('__PRECACHE_ASSETS__', JSON.stringify(manifest.assets));
  await writeFile(join(output, 'sw.js'), worker);
  await writeFile(join(output, '.nojekyll'), '');
}
export async function checkWasm() {
  await buildWasm();
  await mkdir(join(root, 'artifacts/tests'), { recursive: true });
  const temp = await mkdtemp(join(root, 'artifacts/tests/wasm-'));
  try {
    const fixture = join(temp, 'fixture.nsketch');
    await command('cargo', [
      'run',
      '--locked',
      '--quiet',
      '-p',
      'sketch-ffi',
      '--example',
      'portable-fixture',
      '--',
      fixture,
    ]);
    await command(process.execPath, [
      'crates/sketch-wasm/tests/conformance.ts',
      'artifacts/wasm-legacy/sketch_wasm.wasm',
      fixture,
    ]);
    await command(process.execPath, [
      'crates/sketch-wasm/tests/protocol-compatibility.ts',
      'artifacts/wasm-legacy/sketch_wasm.wasm',
    ]);
  } finally {
    await rm(temp, { recursive: true, force: true });
  }
}
export async function checkWeb() {
  await buildWeb();
  await command(process.execPath, ['apps/web/tests/adapter.ts']);
  await command(process.execPath, [
    '--test',
    'scripts/web-tests/output.test.ts',
    'scripts/web-tests/asset-manifest.test.ts',
  ]);
}
