import type { Plugin } from 'vite';

/** Capture Rolldown's complete output graph rather than guessing entry-point filenames. */
export function emittedInventory(): Plugin {
  return {
    name: 'native-sketch-emitted-inventory',
    enforce: 'post',
    generateBundle(_options, bundle) {
      const outputs = Object.values(bundle).sort((a, b) => a.fileName.localeCompare(b.fileName));
      const modules = outputs.flatMap((output) =>
        output.type === 'chunk'
          ? [
              {
                file: output.fileName,
                imports: [...output.imports, ...output.dynamicImports].sort(),
              },
            ]
          : [],
      );
      this.emitFile({
        type: 'asset',
        fileName: 'module-inventory.json',
        source: JSON.stringify(
          { files: outputs.map((output) => output.fileName), modules },
          null,
          2,
        ),
      });
    },
  };
}
