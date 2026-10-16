import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, symlink, rm, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { prepareOutput } from './output.ts';
await test('evidence outputs reject symlink escapes and preserve earlier attempts', async () => {
  const base = await mkdtemp(join(tmpdir(), 'sketch-browser-output-'));
  const root = join(base, 'project');
  await mkdir(join(root, 'artifacts'), { recursive: true });
  await mkdir(join(base, 'outside'));
  try {
    await symlink(join(base, 'outside'), join(root, 'artifacts', 'escape'));
    await assert.rejects(prepareOutput(root, join(root, 'artifacts', 'escape', 'run')), /escapes/);
    const output = join(root, 'artifacts', 'attempt-1');
    await prepareOutput(root, output);
    await writeFile(join(output, 'sentinel'), 'original failure');
    await assert.rejects(prepareOutput(root, output));
    assert.equal(await readFile(join(output, 'sentinel'), 'utf8'), 'original failure');
  } finally {
    await rm(base, { recursive: true, force: true });
  }
});
