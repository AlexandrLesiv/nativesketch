import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, writeFile, rm } from 'node:fs/promises';
import { resolve } from 'node:path';
import { sourceIdentity } from '../evidence/identity.ts';
import { run } from '../lib/process.ts';
const scratch = resolve('artifacts/evidence-tests');
await test('Node discovers executed names, preserves skips and failures without source scanning', async () => {
  await mkdir(scratch, { recursive: true });
  const dir = await mkdtemp(resolve(scratch, 'node-'));
  try {
    const path = resolve(dir, 'fixture.ts');
    await writeFile(
      path,
      `import test from 'node:test';
      for (const name of ['dynamic-pass', 'dynamic-fail']) test(name, () => { if(name.endsWith('fail')) throw Error('expected'); });
      test.skip('skipped', () => {});`,
    );
    const runner = new URL('../evidence/node.ts', import.meta.url).href;
    const output = await run(
      process.execPath,
      [
        '--input-type=module',
        '-e',
        `delete process.env.NODE_TEST_CONTEXT; const {executeNode} = await import(${JSON.stringify(runner)}); console.log(JSON.stringify(await executeNode(${JSON.stringify(dir)}, [${JSON.stringify(path)}])));`,
      ],
      { cwd: dir, capture: true, env: { NODE_TEST_CONTEXT: undefined }, timeout: 10000 },
    );
    const evidence = JSON.parse(output) as {
      tests: {
        id: string;
      }[];
      results: {
        status: string;
      }[];
    };
    assert.deepEqual(
      evidence.tests.map((t) => t.id),
      ['node:fixture.ts:dynamic-pass', 'node:fixture.ts:dynamic-fail', 'node:fixture.ts:skipped'],
    );
    assert.deepEqual(
      evidence.results.map((r) => r.status),
      ['passed', 'failed', 'skipped'],
    );
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});
await test('source identity includes untracked source and ignores disposable artifacts', async () => {
  await mkdir(scratch, { recursive: true });
  const dir = await mkdtemp(resolve(scratch, 'identity-'));
  const git = (args: string[]) => run('git', args, { cwd: dir, capture: true });
  try {
    await git(['init']);
    await writeFile(resolve(dir, '.gitignore'), 'artifacts/\n');
    await git(['add', '.gitignore']);
    await git([
      '-c',
      'user.name=Fixture',
      '-c',
      'user.email=fixture@example.invalid',
      'commit',
      '-m',
      'fixture',
    ]);
    const clean = await sourceIdentity(dir);
    assert.equal(clean.dirtyDiffSha256, null);
    await writeFile(resolve(dir, 'new.ts'), 'a');
    const first = await sourceIdentity(dir);
    await writeFile(resolve(dir, 'new.ts'), 'b');
    assert.notEqual(first.dirtyDiffSha256, (await sourceIdentity(dir)).dirtyDiffSha256);
    await rm(resolve(dir, 'new.ts'));
    await mkdir(resolve(dir, 'artifacts'));
    await writeFile(resolve(dir, 'artifacts/output'), 'ignored');
    assert.deepEqual(await sourceIdentity(dir), clean);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

await test('failed suite cleanup invalidates passing leaf evidence', async () => {
  await mkdir(scratch, { recursive: true });
  const dir = await mkdtemp(resolve(scratch, 'hook-'));
  try {
    const path = resolve(dir, 'fixture.ts');
    await writeFile(
      path,
      `import {describe,it,after} from 'node:test';
      describe('suite',()=>{it('leaf',()=>{});after(()=>{throw Error('cleanup failed')})});`,
    );
    const { executeNode } = await import('../evidence/node.ts');
    const evidence = await executeNode(dir, [path]);
    assert.equal(evidence.results.length, 1);
    assert.equal(evidence.results[0].environment.valid, false);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

await test('Node process and top-level failures cannot certify earlier passing tests', async () => {
  await mkdir(scratch, { recursive: true });
  const dir = await mkdtemp(resolve(scratch, 'fatal-'));
  try {
    const { executeNode } = await import('../evidence/node.ts');
    for (const tail of [
      'throw Error("top-level")',
      'process.exitCode = 1',
      'setTimeout(() => {throw Error("uncaught")}, 10)',
    ]) {
      const path = resolve(dir, 'fixture.ts');
      await writeFile(path, `import test from 'node:test'; await test('leaf',()=>{}); ${tail};`);
      await assert.rejects(executeNode(dir, [path]), /failed before discovery|did not finish/);
    }
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});
