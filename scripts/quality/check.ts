import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
import { checkSourceSize, isSource } from './source-size.ts';
import { checkDependencies } from './dependencies.ts';
import { checkNativeBoundary, isNativeBoundaryPath } from './native-boundary.ts';
import { checkLegacyGrowth, legacyChanges } from './legacy-growth.ts';

const root = fileURLToPath(new URL('../../', import.meta.url));
const args = process.argv.slice(2);
if (args.length > 1 || (args.length && args[0] !== '--final')) {
  throw new Error('Usage: node scripts/quality/check.ts [--final]');
}

const json = (path: string) => JSON.parse(readFileSync(resolve(root, path), 'utf8'));

const run = (executable: string, argv: string[]) =>
  execFileSync(executable, argv, {
    cwd: root,
    encoding: 'utf8',
    timeout: 120_000,
    maxBuffer: 16 * 1024 * 1024,
  });
const paths = [
  ...new Set(
    run('git', ['ls-files', '-z', '--cached', '--others', '--exclude-standard'])
      .split('\0')
      .filter(Boolean),
  ),
];
const deleted = new Set(run('git', ['ls-files', '-z', '--deleted']).split('\0'));
const policy = json('quality/source-policy.json');
const files = paths
  .filter(
    (path) => !deleted.has(path) && (isSource(path, '', policy) || path.startsWith('scripts/')),
  )
  .map((path) => ({ path, text: readFileSync(resolve(root, path), 'utf8') }));
const baseline = json(policy.baseline);
const board = json('tasks/board.json');
const size = checkSourceSize(
  files,
  baseline.entries,
  policy,
  new Set(board.tasks.map((task: { id: string }) => task.id)),
  args[0] === '--final',
);
const metadata = JSON.parse(
  run('cargo', ['metadata', '--locked', '--no-deps', '--format-version', '1']),
);
const legacy = baseline.oversized.map((entry: { path: string }) => entry.path);
const growth = legacyChanges((...argv) => run('git', argv), baseline.revision, legacy, files);
const failures = [
  ...checkNativeBoundary(
    new Map(
      paths
        .filter((path) => !deleted.has(path) && isNativeBoundaryPath(path))
        .map((path) => [path, readFileSync(resolve(root, path), 'utf8')]),
    ),
  ),
  ...size.failures,
  ...checkLegacyGrowth(growth, policy.maximumPhysicalLines),
  ...checkDependencies(metadata, json('quality/dependency-policy.json')),
];
for (const entry of size.debt)
  console.log(`DEBT task ${entry.task}: ${entry.path} (${entry.lines} lines)`);
for (const failure of failures) console.error(`FAIL: ${failure}`);
if (failures.length) process.exitCode = 1;
else
  console.log(
    `PASS: source-size migration gate and production dependency boundaries (${size.debt.length} legacy files remain)`,
  );
