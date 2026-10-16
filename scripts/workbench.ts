import { once } from 'node:events';
import { mkdir, writeFile, access } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { root, main, controller } from './lib/context.ts';
import { workbenchOptions } from './lib/workbench-options.ts';
import { collectSnapshot, startWorkbench, exportWorkbench } from './workbench/index.ts';

await main(async () => {
  if (process.versions.node !== '24.22.0') throw new Error('Use pinned Node 24.22.0');
  const options = workbenchOptions(process.argv.slice(2), root);
  const assets = join(root, 'apps/workbench/dist');
  if (options.mode !== 'snapshot') {
    try {
      await access(join(assets, 'assets/workbench.js'));
      await access(join(assets, 'assets/workbench.css'));
    } catch {
      throw new Error('Workbench assets missing. Run npm run workbench:build first.');
    }
  }
  if (options.mode === 'serve') {
    const service = await startWorkbench({ root, assets, port: options.port });
    try {
      console.log(`Native Sketch project workbench: ${service.url}`);
      console.log('Background actions only. Foreground execution is disabled. Stop with Ctrl+C.');
      if (!controller.signal.aborted) await once(controller.signal, 'abort');
    } finally {
      await service.close();
    }
    return;
  }
  const snapshot = await collectSnapshot(root);
  controller.signal.throwIfAborted();
  await mkdir(dirname(options.output), { recursive: true });
  if (options.mode === 'snapshot')
    await writeFile(options.output, `${JSON.stringify(snapshot, null, 2)}\n`, { flag: 'wx' });
  else await exportWorkbench(snapshot, assets, options.output);
  console.log(`Saved ${options.mode}: ${options.output}`);
});
