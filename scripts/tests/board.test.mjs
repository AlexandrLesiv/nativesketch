import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, writeFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { root } from '../lib/context.mjs';
import { readBoard } from '../lib/board.mjs';
await test('board rejects malformed dependencies and preserves acceptance lifecycle', async (t) => {
  await mkdir(join(root, 'artifacts/tests'), { recursive: true });
  const directory = await mkdtemp(join(root, 'artifacts/tests/board-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  await mkdir(join(directory, 'tasks'));
  await mkdir(join(directory, '.codex/agents'), { recursive: true });
  await writeFile(join(directory, '.codex/agents/worker.toml'), '');
  await writeFile(join(directory, 'tasks/task.md'), 'fixture');
  const first = {
    id: '001',
    title: 'first',
    status: 'review',
    brief: 'tasks/task.md',
    depends_on: [],
    roles: ['worker'],
  };
  const second = { ...first, id: '002', status: 'waiting', depends_on: ['001'] };
  /** @param {unknown[]} tasks */
  const write = (tasks) =>
    writeFile(join(directory, 'tasks/board.json'), JSON.stringify({ schema_version: 1, tasks }));
  await write([first, second]);
  assert.equal((await readBoard(directory)).length, 2);
  await write([first, { ...second, status: 'active' }]);
  await assert.rejects(readBoard(directory), /unfinished dependencies/);
  await write([{ ...first, depends_on: ['002'] }, second]);
  await assert.rejects(readBoard(directory), /cycle/);
  await write([first, { ...second, depends_on: ['999'] }]);
  await assert.rejects(readBoard(directory), /Unknown dependency/);
  await write([first, { ...second, brief: '../outside.md' }]);
  await assert.rejects(readBoard(directory));
});
