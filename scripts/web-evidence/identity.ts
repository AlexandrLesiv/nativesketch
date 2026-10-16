import { rendererIdentity } from './renderer.ts';
import { createHash } from 'node:crypto';
import { join } from 'node:path';
import { treeHash } from '../browser-e2e/identity.ts';
import { fileHash, sourceIdentity } from '../evidence/identity.ts';

/** Bind direct TypeScript consumers to the production Wasm and their complete test tree. */
export async function adapterIdentity(root: string) {
  const wasmSha256 = await fileHash(join(root, 'artifacts/web/sketch_app_wasm.wasm'));
  const adapterSha256 = await treeHash(join(root, 'apps/web/src'));
  const lockSha256 = await fileHash(join(root, 'package-lock.json'));
  const renderer = await rendererIdentity(root);
  const binarySha256 = createHash('sha256')
    .update(JSON.stringify({ wasmSha256, adapterSha256, lockSha256, renderer }))
    .digest('hex');
  const testBinarySha256 = createHash('sha256')
    .update(await treeHash(join(root, 'apps/web/tests')))
    .update(await treeHash(join(root, 'scripts/web-evidence')))
    .update(await fileHash(join(root, 'scripts/test-web-adapter.ts')))
    .update(await fileHash(join(root, 'scripts/evidence/node.ts')))
    .update(await fileHash(join(root, 'scripts/evidence/node-reporter.ts')))
    .digest('hex');
  return {
    source: await sourceIdentity(root),
    renderer,
    wasmSha256,
    adapterSha256,
    lockSha256,
    binarySha256,
    testBinarySha256,
  };
}
