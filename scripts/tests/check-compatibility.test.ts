import test from 'node:test';
import assert from 'node:assert/strict';
import { chmod, copyFile, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, delimiter } from 'node:path';
import { root } from '../lib/context.ts';
import { run } from '../lib/process.ts';

await test('check compatibility name forwards arguments and failures outside its checkout', async (t) => {
  const fixture = await mkdtemp(join(tmpdir(), 'sketch-check-entry-'));
  t.after(() => rm(fixture, { recursive: true }));
  await copyFile(join(root, 'scripts/check'), join(fixture, 'check'));
  await chmod(join(fixture, 'check'), 0o755);
  await writeFile(join(fixture, 'package.json'), '{"type":"module"}');
  await writeFile(
    join(fixture, 'check.ts'),
    `
    console.log(JSON.stringify(process.argv.slice(2)));
    if (process.argv.includes('--fixture-fail')) process.exitCode = 17;
  `,
  );
  const args = ['--core', 'space and ; punctuation', 'Україна'];
  const options = {
    cwd: tmpdir(),
    capture: true,
    env: { PATH: dirname(process.execPath) + delimiter + (process.env.PATH ?? '') },
  };
  const launch = (values: string[]) =>
    process.platform === 'win32'
      ? run(process.execPath, [join(fixture, 'check'), ...values], options)
      : run(join(fixture, 'check'), values, options);
  assert.deepEqual(JSON.parse(await launch(args)), args);
  await assert.rejects(launch(['--fixture-fail']), /exit 17/);
});
