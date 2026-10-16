import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
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
  return {
    version: metadata.version,
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
  if (/\/skia-(raster|precision)\.test\.ts:/.test(id))
    return {
      level: 'rendering',
      inputRoute:
        'Actual production Rust/Wasm scenes and CanvasKit CPU raster; no WebGL, browser or physical input',
    };
  return {
    level: 'bridge',
    inputRoute: id.includes('/injected-failures.test.ts:')
      ? 'TypeScript adapter with injected transport failure; no browser or physical input'
      : 'Actual production Rust/Wasm via TypeScript adapter; no browser or physical input',
  };
}
