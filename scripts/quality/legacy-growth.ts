import { physicalLines } from './source-size.ts';

export type Change = { path: string; before: number; after: number; revision: string };

export function checkLegacyGrowth(changes: Change[], maximum: number) {
  const ceilings: Map<string, number> = new Map();

  const latest: Map<string, Change> = new Map();
  for (const change of changes) {
    ceilings.set(
      change.path,
      Math.max(
        maximum,
        Math.min(ceilings.get(change.path) ?? Infinity, change.before, change.after),
      ),
    );
    latest.set(change.path, change);
  }
  return [...latest.values()]
    .filter((change) => change.after > (ceilings.get(change.path) ?? maximum))
    .map(
      (change) =>
        `${change.path}: oversized source grew ${ceilings.get(change.path)} -> ${change.after} at ${change.revision}`,
    );
}

/** Derive a minimum size ceiling from descendants of the approved baseline.
  Check the current tree against it: repaired regressions can pass again. Exclude
  unrelated merge parents, including the synthetic main parent of a PR merge. */
export function legacyChanges(
  git: (...args: string[]) => string,
  baselineRevision: string,
  paths: string[],
  workingFiles: {
    path: string;
    text: string;
  }[],
): Change[] {
  if (!paths.length) return [];
  git('merge-base', '--is-ancestor', baselineRevision, 'HEAD');

  const counts = (revision: string) => {
    const existing = git('ls-tree', '-rz', '--name-only', revision, '--', ...paths)
      .split('\0')
      .filter(Boolean);
    return new Map(
      existing.map((path) => [path, physicalLines(git('show', `${revision}:${path}`))]),
    );
  };

  const changes: Change[] = [];

  const compare = (before: Map<string, number>, after: Map<string, number>, revision: string) => {
    for (const path of paths) {
      changes.push({ path, before: before.get(path) ?? 0, after: after.get(path) ?? 0, revision });
    }
  };
  const initial = counts(baselineRevision);
  const commits = git(
    'log',
    '--full-history',
    '--ancestry-path',
    '--format=%H',
    `${baselineRevision}..HEAD`,
    '--',
    ...paths,
  )
    .trim()
    .split('\n')
    .filter(Boolean);
  for (const commit of commits) compare(initial, counts(commit), commit);
  const working = new Map(workingFiles.map((file) => [file.path, physicalLines(file.text)]));
  compare(initial, working, 'working-tree');
  return changes;
}
