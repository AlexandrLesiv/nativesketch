import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { command, root } from '../lib/context.ts';

/** Keep browser and Node type projects separate for the type-aware lint engine. */
export async function lintTypeScript() {
  const webTools = (await readdir(join(root, 'apps/web')))
    .filter((name) => name.endsWith('.ts'))
    .map((name) => `apps/web/${name}`);
  for (const [config, paths] of [
    [
      'tsconfig.scripts.json',
      [
        'scripts',
        ...webTools,
        'apps/web/tests',
        'crates/sketch-wasm/tests',
        'crates/sketch-app-ffi/tests',
        'crates/sketch-app-wasm/tests',
      ],
    ],
    ['apps/web/tsconfig.json', ['apps/web/src']],
    ['apps/web/e2e/tsconfig.json', ['apps/web/e2e']],
    ['apps/workbench/tsconfig.json', ['apps/workbench/src']],
    ['apps/workbench/e2e/tsconfig.json', ['apps/workbench/e2e', 'apps/workbench/vite.config.ts']],
  ] as const) {
    await command(process.execPath, [
      'node_modules/oxlint/bin/oxlint',
      '--type-aware',
      '--tsconfig',
      config,
      ...paths,
    ]);
  }
}
