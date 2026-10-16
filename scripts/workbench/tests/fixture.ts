import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
export async function fixture() {
  const root = await mkdtemp(join(tmpdir(), 'native-sketch-workbench-'));
  const put = async (path: string, content: string) => {
    await mkdir(join(root, path, '..'), { recursive: true });
    await writeFile(join(root, path), content);
  };
  await put('.gitignore', 'artifacts/\n');
  await put('package.json', '{"type":"module"}');
  await put('package-lock.json', '{"packages":{"":{},"node_modules/fixture":{"version":"1.0.0"}}}');
  await put('.codex/agents/test.toml', 'name="test"');
  await put(
    'tasks/board.json',
    JSON.stringify({
      schema_version: 1,
      tasks: [
        {
          id: '001',
          title: 'Fixture',
          status: 'waiting',
          brief: 'tasks/001.md',
          depends_on: [],
          roles: ['test'],
        },
      ],
    }),
  );
  await put('tasks/001.md', '# Fixture\nNo native UI.');
  await put('docs/example.md', '# Example\n<script>alert(1)</script>\n/Users/private/secret');
  await put(
    'quality/source-policy.json',
    JSON.stringify({
      maximumPhysicalLines: 200,
      extensions: ['.ts'],
      excludedPrefixes: [],
      baseline: 'unused',
    }),
  );
  await put(
    'quality/features/fixture.json',
    JSON.stringify({
      schemaVersion: 1,
      id: 'fixture',
      description: 'Fixture',
      sourceRevision: 'fixture',
      cases: [
        {
          id: 'FIX-001',
          expected: 'Must run',
          invariants: [],
          ownerTask: '001',
          requirements: [{ platform: 'portable', level: 'unit' }],
          testIds: [],
          automation: 'planned',
          gaps: [{ reason: 'Not implemented', ownerTask: '001' }],
        },
      ],
    }),
  );
  await put('scripts/check.ts', 'console.error("disposable failure"); process.exit(7);');
  await put('artifacts/ui/index.html', '<!doctype html><div id="workbench-root"></div>');
  await put('artifacts/ui/assets/workbench.js', 'globalThis.fixture=true;');
  await put('artifacts/ui/assets/workbench.css', 'body {color: black;}');
  const git = (args: string[]) => promisify(execFile)('git', args, { cwd: root });
  await git(['init', '-q']);
  await git(['add', '.']);
  await git([
    '-c',
    'user.name=Fixture',
    '-c',
    'user.email=fixture@example.invalid',
    'commit',
    '-qm',
    'fixture',
  ]);
  return {
    root,
    put,
    assets: join(root, 'artifacts/ui'),
    close: () => rm(root, { recursive: true, force: true }),
  };
}
