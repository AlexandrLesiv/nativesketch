import { physicalLines } from './source-size.mjs';

/** @typedef {{path: string, before: number, after: number, revision: string}} Change */
/** @param {Change[]} changes @param {number} maximum */
export function checkLegacyGrowth(changes, maximum) {
  return changes.filter((change) => change.after > maximum && change.after > change.before)
    .map((change) => `${change.path}: oversized source grew ${change.before} -> ${change.after} at ${change.revision}`);
}

/**
 * Check each first-parent commit since the approved baseline and the working tree.
 * A shrinking legacy file cannot later regrow under its original baseline ceiling.
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
  const commits = git('log', '--first-parent', '--format=%H', `${baselineRevision}..HEAD`, '--', ...paths)
    .trim().split('\n').filter(Boolean);
  for (const commit of commits) compare(counts(`${commit}^`), counts(commit), commit);
  const working = new Map(workingFiles.map((file) => [file.path, physicalLines(file.text)]));
  compare(counts('HEAD'), working, 'working-tree');
  return changes;
}
