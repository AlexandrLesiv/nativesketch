import test from 'node:test';
import assert from 'node:assert/strict';
import { join } from 'node:path';
import { command, root } from '../lib/context.mjs';
await test('command preserves repository Cargo output when adding native tool environment', async () => {
  const previous = process.env.CARGO_TARGET_DIR;
  try {
    process.env.CARGO_TARGET_DIR = '/external/stale-build';
    const output = await command(
      process.execPath,
      [
        '-e',
        'console.log(JSON.stringify([process.env.CARGO_TARGET_DIR,process.env.SKETCH_RUST_PROFILE]))',
      ],
      { env: { SKETCH_RUST_PROFILE: 'release' }, capture: true },
    );
    assert.deepEqual(JSON.parse(String(output)), [join(root, 'target'), 'release']);
  } finally {
    if (previous === undefined) delete process.env.CARGO_TARGET_DIR;
    else process.env.CARGO_TARGET_DIR = previous;
  }
});
await test('explicit host Cargo output remains supported', async () => {
  const path = join(root, 'target/linux');
  const output = await command(
    process.execPath,
    ['-e', 'console.log(process.env.CARGO_TARGET_DIR)'],
    { env: { CARGO_TARGET_DIR: path }, capture: true },
  );
  assert.equal(output, path);
});
