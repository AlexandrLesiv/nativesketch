import { readFile, readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { join } from 'node:path';

function record(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value))
    throw new Error('Invalid emitted module inventory');
  return value as Record<string, unknown>;
}
function paths(value: unknown): string[] {
  if (!Array.isArray(value) || !value.every((path: unknown) => typeof path === 'string'))
    throw new Error('Invalid emitted paths');
  const result: string[] = value;
  for (const path of result)
    if (
      !/^[\w.-]+(?:\/[\w.-]+)*$/.test(path) ||
      path.split('/').some((p) => p === '.' || p === '..')
    )
      throw new Error(`Unsafe emitted path: ${path}`);
  if (new Set(result).size !== result.length) throw new Error('Duplicate emitted paths');
  return result;
}
async function diskFiles(directory: string, prefix = ''): Promise<string[]> {
  const result: string[] = [];
  for (const item of await readdir(join(directory, prefix), { withFileTypes: true })) {
    const relative = prefix + item.name;
    if (item.isDirectory()) result.push(...(await diskFiles(directory, `${relative}/`)));
    else if (item.isFile()) result.push(relative);
    else throw new Error(`Unsupported output entry: ${relative}`);
  }
  return result.sort();
}

/** Hash and precache every emitted dependency. A missing/stale output fails the build. */
export async function webAssetManifest(directory: string) {
  const inventory = record(
    JSON.parse(await readFile(join(directory, 'module-inventory.json'), 'utf8')),
  );
  const emitted = paths(inventory.files);
  if (!Array.isArray(inventory.modules)) throw new Error('Missing emitted module graph');
  const modules = inventory.modules.map((value: unknown) => {
    const module = record(value);
    return { file: paths([module.file])[0], imports: paths(module.imports) };
  });
  const moduleNames = new Set(modules.map((module) => module.file));
  if (
    moduleNames.size !== modules.length ||
    !moduleNames.has('main.js') ||
    !moduleNames.has('engine.js')
  )
    throw new Error('Incomplete emitted entry-point graph');
  for (const module of modules) {
    if (!emitted.includes(module.file)) throw new Error(`Unlisted module: ${module.file}`);
    for (const imported of module.imports)
      if (!moduleNames.has(imported)) throw new Error(`Unlisted module dependency: ${imported}`);
  }
  if (emitted.some((file) => file.endsWith('.js') && !moduleNames.has(file)))
    throw new Error('Emitted JavaScript missing from module graph');
  const files = paths([
    'index.html',
    'style.css',
    'sketch_app_wasm.wasm',
    'module-inventory.json',
    ...emitted,
  ]).sort();
  // The service worker and Pages marker are generated only after this verified payload.
  const disk = (await diskFiles(directory)).filter(
    (file) => file !== 'sw.js' && file !== '.nojekyll',
  );
  if (JSON.stringify(disk) !== JSON.stringify(files))
    throw new Error('Emitted asset inventory does not match output files');
  const hash = createHash('sha256');
  for (const file of files) {
    const bytes = await readFile(join(directory, file));
    hash.update(JSON.stringify([file, bytes.length])).update(bytes);
  }
  return {
    files,
    assets: ['./', ...files.map((file) => `./${file}`)],
    version: hash.digest('hex').slice(0, 16),
  };
}
