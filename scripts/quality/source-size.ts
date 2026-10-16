import { extname } from 'node:path';

export function physicalLines(text: string) {
  return text.length === 0 ? 0 : text.split('\n').length - Number(text.endsWith('\n'));
}

export type BaselineEntry = {
  path: string;
  category: string;
  physicalLines: number;
  extractionTask?: string;
};
export type SourcePolicy = {
  maximumPhysicalLines: number;
  sourceExtensions: string[];
  excludedSources: { path: string; reason: string }[];
};

export function isSource(path: string, text: string, policy: SourcePolicy) {
  return (
    policy.sourceExtensions.includes(extname(path)) ||
    (path.startsWith('scripts/') && text.startsWith('#!'))
  );
}

export function checkSourceSize(
  files: {
    path: string;
    text: string;
  }[],
  baseline: BaselineEntry[],
  policy: SourcePolicy,
  taskIds: Set<string>,
  final: boolean = false,
) {
  const previous = new Map(baseline.map((entry) => [entry.path, entry]));

  const failures: string[] = [];

  const debt: {
    path: string;
    lines: number;
    task: string;
  }[] = [];
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
    if (
      !final &&
      old?.category === 'handwritten-source' &&
      task &&
      taskIds.has(task) &&
      lines <= old.physicalLines
    ) {
      debt.push({ path: file.path, lines, task });
    } else {
      failures.push(`${file.path}: ${lines} lines exceeds ${policy.maximumPhysicalLines}`);
    }
  }
  return { failures, debt };
}
