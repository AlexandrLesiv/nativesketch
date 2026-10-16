import test from 'node:test';
import assert from 'node:assert/strict';
import { chmod, readFile, symlink } from 'node:fs/promises';
import { join } from 'node:path';
import { fixture } from './fixture.ts';
import { dependencies } from '../adapters/dependencies.ts';
import { coverage } from '../adapters/coverage.ts';
import { sourceIdentity } from '../identity.ts';
import { SnapshotState } from '../state.ts';
import { startWorkbench } from '../service.ts';
await test('workbench dependency refresh disables toolchain auto-install', async () => {
  const f = await fixture();
  const previous = process.env.PATH;
  try {
    await f.put(
      'artifacts/bin/cargo',
      `#!${process.execPath}\nif(process.env.RUSTUP_AUTO_INSTALL!=='0') process.exit(91);\nconsole.log(JSON.stringify({packages:[],resolve:{nodes:[]},workspace_members:[]}));`,
    );
    await chmod(join(f.root, 'artifacts/bin/cargo'), 0o755);
    process.env.PATH = join(f.root, 'artifacts/bin');
    const result = await dependencies(f.root);
    assert.equal(result.state, 'ready');
    assert.ok(!result.diagnostics.some((item) => item.code === 'rust-inventory-unavailable'));
  } finally {
    process.env.PATH = previous;
    await f.close();
  }
});
await test('workbench rejects aggregate feature bounds and malformed partial evidence pairs', async () => {
  const f = await fixture();
  try {
    const source = await sourceIdentity(f.root);
    const feature = await readFile(join(f.root, 'quality/features/fixture.json'), 'utf8');
    const paths: string[] = [];
    for (let i = 0; i < 7; i++) {
      const path = `quality/features/large-${i}.json`;
      await f.put(path, feature + ' '.repeat(1900000));
      paths.push(path);
    }
    assert.equal((await coverage(f.root, paths, source)).state, 'invalid');
    await f.put('artifacts/evidence/discovery.json', '{}');
    assert.equal(
      (await coverage(f.root, ['quality/features/fixture.json'], source)).state,
      'invalid',
    );
  } finally {
    await f.close();
  }
});
await test('workbench failed refresh retains previous snapshot as stale', async () => {
  const f = await fixture();
  const state = new SnapshotState(f.root);
  try {
    const first = await state.refresh();
    await symlink('/etc/passwd', join(f.root, 'escape.txt'));
    const failed = await state.refresh();
    assert.equal(failed.stale, true);
    assert.deepEqual(failed.snapshot, first.snapshot);
    assert.equal(failed.diagnostics[0]?.code, 'refresh-failed');
  } finally {
    await state.close();
    await f.close();
  }
});
await test('workbench storage rejects escaped project directories', async () => {
  const f = await fixture();
  const outside = await fixture();
  try {
    await symlink(outside.root, join(f.root, 'artifacts/workbench'));
    await assert.rejects(startWorkbench({ root: f.root, assets: f.assets }), /Unsafe storage/);
  } finally {
    await f.close();
    await outside.close();
  }
});
