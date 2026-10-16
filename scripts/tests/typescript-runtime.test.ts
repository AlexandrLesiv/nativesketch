import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, writeFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { root, command } from '../lib/context.ts';

await test('native TypeScript runs erased types; compiler rejects incompatible assignments', async () => {
  await mkdir(join(root, 'artifacts/tests'), { recursive: true });
  const directory = await mkdtemp(join(root, 'artifacts/tests/typescript-'));
  const fixture = join(directory, 'fixture.ts');
  try {
    await writeFile(join(directory, 'contract.ts'), 'export interface Count { value: number }\n');
    await writeFile(
      fixture,
      `import type { Count } from './contract.ts';
      const count: Count = {value: 3}; console.log(count.value);`,
    );
    assert.equal(await command(process.execPath, [fixture], { capture: true }), '3');
    const config = join(directory, 'tsconfig.json');
    await writeFile(
      config,
      JSON.stringify({ extends: join(root, 'tsconfig.scripts.json'), include: ['./*.ts'] }),
    );
    const check = () =>
      command(process.execPath, ['node_modules/typescript/bin/tsc', '--project', config], {
        capture: true,
      });
    await check();
    await writeFile(
      fixture,
      `import type { Count } from './contract.ts';
      const count: Count = {value: 'incorrect'}; console.log(count.value);`,
    );
    await assert.rejects(check(), /TS2322/);
    await writeFile(fixture, 'enum Unsupported { Value }\nconsole.log(Unsupported.Value);');
    await assert.rejects(
      command(process.execPath, [fixture], { capture: true }),
      /ERR_UNSUPPORTED_TYPESCRIPT_SYNTAX/,
    );
    await assert.rejects(check(), /TS1294/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
