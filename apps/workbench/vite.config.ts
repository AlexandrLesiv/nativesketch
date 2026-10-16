import { defineConfig } from 'vite';
import { fileURLToPath } from 'node:url';
export default defineConfig({
  root: fileURLToPath(new URL('.', import.meta.url)),
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    lib: {
      entry: fileURLToPath(new URL('src/main.ts', import.meta.url)),
      name: 'NativeSketchWorkbench',
      formats: ['iife'],
      fileName: () => 'assets/workbench.js',
      cssFileName: 'assets/workbench',
    },
  },
  plugins: [
    {
      name: 'workbench-shell',
      generateBundle() {
        this.emitFile({
          type: 'asset',
          fileName: 'index.html',
          source:
            '<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Native Sketch · Project workbench</title><link rel="stylesheet" href="/assets/workbench.css"></head><body><div id="workbench-root"></div><script id="workbench-bootstrap" type="application/json">{"mode":"live"}</script><script src="/assets/workbench.js" defer></script></body></html>',
        });
      },
    },
  ],
});
