import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { command, root } from './context.ts';
import { lintTypeScript } from '../quality/typescript.ts';
export async function checkTooling() {
  if (process.versions.node !== '24.22.0') throw new Error('Use pinned Node 24.22.0');
  await command(process.execPath, [
    'node_modules/typescript/bin/tsc',
    '-p',
    'tsconfig.scripts.json',
  ]);
  await command(process.execPath, [
    'node_modules/typescript/bin/tsc',
    '-p',
    'apps/web/e2e/tsconfig.json',
  ]);
  for (const config of ['apps/workbench/tsconfig.json', 'apps/workbench/e2e/tsconfig.json'])
    await command(process.execPath, ['node_modules/typescript/bin/tsc', '-p', config]);
  await lintTypeScript();
  await command(process.execPath, [
    'node_modules/oxfmt/bin/oxfmt',
    '--check',
    'scripts/**/*.ts',
    'scripts/*.ts',
    'apps/web/*.ts',
    'apps/web/src/*.ts',
    'apps/web/tests/*.ts',
    'apps/web/e2e/*.ts',
    'crates/sketch-wasm/tests/*.ts',
    'crates/sketch-app-ffi/tests/*.ts',
    'crates/sketch-app-wasm/tests/*.ts',
    'apps/workbench/**/*.ts',
    'apps/workbench/**/*.css',
  ]);
  const tests = [];
  for (const directory of [
    'scripts/tests',
    'scripts/quality',
    'scripts/browser-e2e',
    'scripts/reliability',
    'scripts/workbench/tests',
  ]) {
    for (const name of await readdir(join(root, directory)))
      if (name.endsWith('.test.ts')) tests.push(`${directory}/${name}`);
  }
  await command(process.execPath, ['--test', ...tests]);
}
