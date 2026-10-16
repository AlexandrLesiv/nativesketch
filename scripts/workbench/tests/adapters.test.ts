import test from 'node:test';
import assert from 'node:assert/strict';
import { symlink, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fixture } from './fixture.ts';
import { boundedRead, safeText } from '../files.ts';
import { sourceIdentity } from '../identity.ts';
import { sourceIdentity as originalIdentity } from '../../evidence/identity.ts';
import { collectSnapshot } from '../snapshot.ts';
import { exportWorkbench } from '../export.ts';
import { readBoard } from '../../lib/board.ts';
import { tasks } from '../adapters/tasks.ts';
import { coverage } from '../adapters/coverage.ts';
await test('workbench snapshot preserves missing coverage and private-path redaction', async () => {
  const f = await fixture();
  try {
    const snapshot = await collectSnapshot(f.root);
    assert.equal(snapshot.tasks.data[0].status, 'waiting');
    assert.notEqual(snapshot.coverage.data.rows[0].status, 'passed');
    assert.equal(snapshot.coverage.data.freshness, 'unknown');
    assert.equal(snapshot.coverage.state, 'missing');
    assert.deepEqual(snapshot.coverage.data.issues, []);
    assert.ok(!JSON.stringify(snapshot).includes('/Users/private'));
    const output = join(f.root, 'artifacts/report.html');
    await exportWorkbench(snapshot, f.assets, output);
    const html = await readFile(output, 'utf8');
    assert.ok(html.includes('"mode":"static"'));
    assert.ok(!html.includes('<script>alert(1)'));
    await assert.rejects(exportWorkbench(snapshot, f.assets, output), /EEXIST/);
  } finally {
    await f.close();
  }
});
await test('workbench bounds identity reads and rejects symlink escapes', async () => {
  const f = await fixture();
  try {
    await f.put('untracked.txt', 'small');
    assert.deepEqual(await sourceIdentity(f.root), await originalIdentity(f.root));
    await symlink('/etc/passwd', join(f.root, 'escape.txt'));
    await assert.rejects(boundedRead(f.root, 'escape.txt'));
    await assert.rejects(sourceIdentity(f.root));
    await f.put('oversized.txt', 'x'.repeat(2 * 1024 * 1024 + 1));
    await assert.rejects(boundedRead(f.root, 'oversized.txt'));
    assert.equal(
      safeText('C:\\Users\\private\\secret.txt /home/private/key'),
      '[private-path] [private-path]',
    );
  } finally {
    await f.close();
  }
});
await test('invalid board retains readable rows while malformed evidence is invalid', async () => {
  const f = await fixture();
  try {
    const board = JSON.parse(await readFile(join(f.root, 'tasks/board.json'), 'utf8'));
    board.tasks[0].depends_on = ['001'];
    await f.put('tasks/board.json', JSON.stringify(board));
    const result = await tasks(f.root);
    assert.equal(result.state, 'invalid');
    assert.equal(result.data[0].id, '001');
    await f.put('artifacts/evidence/discovery.json', '{bad');
    const evidence = await coverage(
      f.root,
      ['quality/features/fixture.json'],
      await sourceIdentity(f.root),
    );
    assert.equal(evidence.state, 'invalid');
  } finally {
    await f.close();
  }
});

await test('bounded supplied board validation matches ordinary malformed and cycle validation', async () => {
  const f = await fixture();
  try {
    const text = await readFile(join(f.root, 'tasks/board.json'), 'utf8');
    assert.deepEqual(await readBoard(f.root), await readBoard(f.root, { text, roles: ['test'] }));
    const board = JSON.parse(text);
    board.tasks[0].depends_on = ['001'];
    for (const bad of ['{broken', JSON.stringify(board)]) {
      await f.put('tasks/board.json', bad);
      await assert.rejects(readBoard(f.root));
      await assert.rejects(readBoard(f.root, { text: bad, roles: ['test'] }));
    }
  } finally {
    await f.close();
  }
});
