import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import type { ProjectSnapshot } from '../../../scripts/workbench/contracts/index.ts';
export async function offlineFile(path: string, snapshot: ProjectSnapshot): Promise<void> {
  const base = resolve(import.meta.dirname, '../dist');
  const js = await readFile(resolve(base, 'assets/workbench.js'), 'utf8');
  const css = await readFile(resolve(base, 'assets/workbench.css'), 'utf8');
  const bootstrap = JSON.stringify({ mode: 'static', snapshot })
    .replaceAll('<', '\\u003c')
    .replaceAll('\u2028', '\\u2028')
    .replaceAll('\u2029', '\\u2029');
  await writeFile(
    path,
    `<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Fixture offline report</title><style>${css}</style><body><div id="workbench-root"></div><script id="workbench-bootstrap" type="application/json">${bootstrap}</script><script>${js.replaceAll('</script', '<\\/script')}</script></body></html>`,
  );
}
