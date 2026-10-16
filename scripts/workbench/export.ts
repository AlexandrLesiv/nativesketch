import { writeFile } from 'node:fs/promises';
import type { ProjectSnapshot, Bootstrap } from './contracts/index.ts';
import { boundedRead } from './files.ts';
export async function exportWorkbench(snapshot: ProjectSnapshot, assets: string, output: string) {
  const js = (await boundedRead(assets, 'assets/workbench.js', 4 * 1024 * 1024)).toString();
  const css = (await boundedRead(assets, 'assets/workbench.css')).toString();
  const bootstrap: Bootstrap = { mode: 'static', snapshot };
  const data = JSON.stringify(bootstrap)
    .replace(/</g, '\\u003c')
    .replace(/\u2028/g, '\\u2028')
    .replace(/\u2029/g, '\\u2029');
  if (Buffer.byteLength(data) > 24 * 1024 * 1024) throw new Error('Export snapshot exceeds bound');
  const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="referrer" content="no-referrer"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; img-src data:; connect-src 'none'; base-uri 'none'"><title>Native Sketch project workbench</title><style>${css.replace(/<\/style/gi, '<\\/style')}</style></head><body><div id="workbench-root"></div><script id="workbench-bootstrap" type="application/json">${data}</script><script>${js.replace(/<\/script/gi, '<\\/script')}</script></body></html>`;
  await writeFile(output, html, { flag: 'wx' });
}
