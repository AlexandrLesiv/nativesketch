import { physicalLines } from './source-size.mjs';

/** @typedef {{path: string, before: number, after: number, revision: string}} Change */
/** @param {Change[]} changes @param {number} maximum */
export function checkLegacyGrowth(changes, maximum) {
  /** @type {Map<string, number>} */
  const ceilings = new Map();
  /** @type {Map<string, Change>} */
  const latest = new Map();
  for (const change of changes) {
    ceilings.set(change.path, Math.max(maximum,
      Math.min(ceilings.get(change.path) ?? Infinity, change.before, change.after)));
    latest.set(change.path, change);
  }
  return [...latest.values()].filter((change) => change.after > (ceilings.get(change.path) ?? maximum))
    .map((change) => `${change.path}: oversized source grew ${ceilings.get(change.path)} -> ${change.after} at ${change.revision}`);
}

/**
 * Derive a minimum size ceiling from descendants of the approved baseline.
 * Check the current tree against it: repaired regressions can pass again. Exclude
 * unrelated merge parents, including the synthetic main parent of a PR merge.
 * @param {(...args: string[]) => string} git
 * @param {string} baselineRevision
 * @param {string[]} paths
 * @param {{path: string, text: string}[]} workingFiles
 * @returns {Change[]}
 */
export function legacyChanges(git, baselineRevision, paths, workingFiles) {
  if (!paths.length) return [];
  git('merge-base', '--is-ancestor', baselineRevision, 'HEAD');
  /** @param {string} revision */
  const counts = (revision) => {
    const existing = git('ls-tree', '-rz', '--name-only', revision, '--', ...paths)
      .split('\0').filter(Boolean);
    return new Map(existing.map((path) => [path, physicalLines(git('show', `${revision}:${path}`))]));
  };
  /** @type {Change[]} */
  const changes = [];
  /** @param {Map<string, number>} before @param {Map<string, number>} after @param {string} revision */
  const compare = (before, after, revision) => {
    for (const path of paths) {
      changes.push({ path, before: before.get(path) ?? 0, after: after.get(path) ?? 0, revision });
    }
  };
  const initial = counts(baselineRevision);
  const commits = git('log', '--full-history', '--ancestry-path', '--format=%H', `${baselineRevision}..HEAD`, '--', ...paths)
    .trim().split('\n').filter(Boolean);
  for (const commit of commits) compare(initial, counts(commit), commit);
  const working = new Map(workingFiles.map((file) => [file.path, physicalLines(file.text)]));
  compare(initial, working, 'working-tree');
  return changes;
}
