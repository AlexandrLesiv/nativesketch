import { extname } from 'node:path';

/** @param {string} text */
export function physicalLines(text) {
  return text.length === 0 ? 0 : text.split('\n').length - Number(text.endsWith('\n'));
}

/** @typedef {{path: string, category: string, physicalLines: number, extractionTask?: string}} BaselineEntry */
/** @typedef {{maximumPhysicalLines: number, sourceExtensions: string[], excludedSources: {path: string, reason: string}[]}} SourcePolicy */

/** @param {string} path @param {string} text @param {SourcePolicy} policy */
export function isSource(path, text, policy) {
  return policy.sourceExtensions.includes(extname(path)) ||
    (path.startsWith('scripts/') && text.startsWith('#!'));
}

/**
 * @param {{path: string, text: string}[]} files
 * @param {BaselineEntry[]} baseline
 * @param {SourcePolicy} policy
 * @param {Set<string>} taskIds
 * @param {boolean} final
 */
export function checkSourceSize(files, baseline, policy, taskIds, final = false) {
  const previous = new Map(baseline.map((entry) => [entry.path, entry]));
  /** @type {string[]} */
  const failures = [];
  /** @type {{path: string, lines: number, task: string}[]} */
  const debt = [];
  for (const exclusion of policy.excludedSources) {
    if (!exclusion.reason.trim()) failures.push(`Unexplained source exclusion: ${exclusion.path}`);
  }
  for (const file of files) {
    if (!isSource(file.path, file.text, policy)) continue;
    if (policy.excludedSources.some((entry) => entry.path === file.path)) continue;
    const lines = physicalLines(file.text);
    if (lines <= policy.maximumPhysicalLines) continue;
    const old = previous.get(file.path);
    const task = old?.extractionTask;
    if (!final && old?.category === 'handwritten-source' &&
        task && taskIds.has(task) && lines <= old.physicalLines) {
      debt.push({ path: file.path, lines, task });
    } else {
      failures.push(`${file.path}: ${lines} lines exceeds ${policy.maximumPhysicalLines}`);
    }
  }
  return { failures, debt };
}
