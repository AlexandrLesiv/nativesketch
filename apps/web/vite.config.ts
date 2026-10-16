import { emittedInventory } from './emitted-inventory.ts';
import { defineConfig } from 'vite';
import { fileURLToPath } from 'node:url';
export default defineConfig({
  root: fileURLToPath(new URL('.', import.meta.url)),
  publicDir: false,
  plugins: [emittedInventory()],
  build: {
    outDir: '../../artifacts/web',
    emptyOutDir: false,
    minify: false,
    lib: { entry: { main: 'src/main.ts', engine: 'src/engine.ts' }, formats: ['es'] },
    rolldownOptions: {
      output: {
        entryFileNames: '[name].js',
        chunkFileNames: '[name].js',
        preserveModules: true,
        preserveModulesRoot: 'src',
      },
    },
  },
});
