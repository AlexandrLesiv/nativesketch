import type { Dependency, Section } from '../contracts/index.ts';
import { run } from '../../lib/process.ts';
import { boundedRead, hash } from '../files.ts';
function record(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value))
    throw new Error('Expected dependency object');
  return value as Record<string, unknown>;
}
export async function dependencies(root: string): Promise<Section<Dependency[]>> {
  const result: Section<Dependency[]> = { state: 'ready', diagnostics: [], data: [] };
  try {
    const metadata = record(
      JSON.parse(
        await run('cargo', ['metadata', '--locked', '--offline', '--format-version', '1'], {
          cwd: root,
          capture: true,
          env: { RUSTUP_AUTO_INSTALL: '0' },
          timeout: 30_000,
        }),
      ),
    );
    if (!Array.isArray(metadata.packages) || metadata.packages.length > 10000)
      throw new Error('Package bound');
    const packages = metadata.packages.map(record);
    const resolve = record(metadata.resolve);
    if (!Array.isArray(resolve.nodes)) throw new Error('Missing resolution');
    const nodes = resolve.nodes.map(record);
    const work = new Set(
      Array.isArray(metadata.workspace_members) ? metadata.workspace_members : [],
    );
    const direct = new Set(
      nodes
        .filter((n) => work.has(n.id))
        .flatMap((n) => (Array.isArray(n.dependencies) ? n.dependencies : [])),
    );
    for (const item of packages) {
      const node = nodes.find((n) => n.id === item.id);
      result.data.push({
        id: `rust:${hash(String(item.id)).slice(0, 20)}`,
        ecosystem: 'Rust',
        name: String(item.name),
        version: String(item.version),
        versionKind: 'resolved',
        scope: direct.has(item.id) || work.has(item.id) ? 'direct' : 'transitive',
        license: typeof item.license === 'string' ? item.license : null,
        provenance: 'Cargo.lock',
        dependencies: (Array.isArray(node?.dependencies) ? node.dependencies : []).map(
          (id: unknown) => `rust:${hash(String(id)).slice(0, 20)}`,
        ),
      });
    }
  } catch {
    result.state = 'partial';
    result.diagnostics.push({
      severity: 'warning',
      code: 'rust-inventory-unavailable',
      path: 'Cargo.lock',
      message: 'Offline locked Cargo inventory unavailable.',
    });
  }
  try {
    const lock = record(
      JSON.parse((await boundedRead(root, 'package-lock.json', 4 * 1024 * 1024)).toString()),
    );
    const packages = record(lock.packages);
    const entries = Object.entries(packages);
    if (entries.length > 10000) throw new Error('Package bound');
    const direct = {
      ...record(record(packages['']).dependencies ?? {}),
      ...record(record(packages['']).devDependencies ?? {}),
    };
    for (const [path, value] of entries) {
      if (!path) continue;
      const item = record(value);
      const name = path.split('node_modules/').at(-1) ?? path;
      result.data.push({
        id: `npm:${path}`,
        ecosystem: 'npm',
        name,
        version: typeof item.version === 'string' ? item.version : null,
        versionKind: typeof item.version === 'string' ? 'resolved' : 'unknown',
        scope: name in direct ? 'direct' : 'transitive',
        license: typeof item.license === 'string' ? item.license : null,
        provenance: 'package-lock.json',
        dependencies: [],
      });
    }
  } catch {
    result.state = 'partial';
    result.diagnostics.push({
      severity: 'warning',
      code: 'npm-inventory-unavailable',
      path: 'package-lock.json',
      message: 'npm lockfile unavailable or invalid.',
    });
  }
  result.diagnostics.push({
    severity: 'info',
    code: 'other-ecosystems',
    message:
      'Swift/Kotlin/.NET resolved inventories and advisory/license results are not inferred from Rust/npm inventory.',
  });
  return result;
}
