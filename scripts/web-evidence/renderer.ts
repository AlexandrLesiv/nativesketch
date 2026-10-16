import { createRequire } from 'node:module';
import { readFile, realpath } from 'node:fs/promises';
import { join, relative } from 'node:path';
import { fileHash } from '../evidence/identity.ts';
import { treeHash } from '../browser-e2e/identity.ts';
export async function rendererIdentity(root: string) {
  const packagePath = join(root, 'node_modules/canvaskit-wasm');
  const metadata: unknown = JSON.parse(await readFile(join(packagePath, 'package.json'), 'utf8'));
  if (
    !metadata ||
    typeof metadata !== 'object' ||
    !('version' in metadata) ||
    metadata.version !== '0.42.0'
  )
    throw Error('Unexpected CanvasKit package version');
  const exports = 'exports' in metadata ? metadata.exports : undefined;
  const route =
    exports && typeof exports === 'object' && './full' in exports ? exports['./full'] : undefined;
  if (
    !route ||
    typeof route !== 'object' ||
    JSON.stringify(Object.keys(route).sort()) !== JSON.stringify(['import', 'require', 'types']) ||
    !('import' in route) ||
    route.import !== './bin/full/canvaskit.js' ||
    !('require' in route) ||
    route.require !== './bin/full/canvaskit.js' ||
    !('types' in route) ||
    route.types !== './types/index.d.ts'
  )
    throw Error('Unexpected CanvasKit full export routing');
  const resolved = await realpath(
    createRequire(join(root, 'package.json')).resolve('canvaskit-wasm/full'),
  );
  if (resolved !== (await realpath(join(packagePath, 'bin/full/canvaskit.js'))))
    throw Error('Unexpected resolved CanvasKit full module');
  return {
    version: metadata.version,
    packageSha256: await fileHash(join(packagePath, 'package.json')),
    resolvedFullModule: relative(root, resolved),
    declaredFullTypes: route.types,
    javascriptSha256: await fileHash(join(packagePath, 'bin/full/canvaskit.js')),
    wasmSha256: await fileHash(join(packagePath, 'bin/full/canvaskit.wasm')),
    declarationsSha256: await treeHash(join(packagePath, 'types')),
    browserTypesSha256: await treeHash(join(root, 'apps/web/types')),
    outputSha256: await treeHash(join(root, 'artifacts/web')),
  };
}
export function testClassification(id: string) {
  if (id.includes('/skia-types.test.ts:'))
    return {
      level: 'tooling',
      inputRoute:
        'TypeScript compiler positive/negative fixtures; no Wasm execution, browser or physical input',
    };
  if (/\/skia-(raster|precision|prepared)\.test\.ts:/.test(id))
    return {
      level: 'rendering',
      inputRoute:
        'Actual production Rust/Wasm scenes and CanvasKit CPU raster; no WebGL, browser or physical input',
    };
  return {
    level: 'bridge',
    inputRoute:
      id.includes('/injected-failures.test.ts:') ||
      (/\/prepared-(ownership|quarantine)\.test\.ts:/.test(id) && id.includes('injected'))
        ? 'TypeScript adapter with injected transport failure; no browser or physical input'
        : 'Actual production Rust/Wasm via TypeScript adapter; no browser or physical input',
  };
}
