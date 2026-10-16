export function requireVersion(actual: string, expected: string) {
  if (!actual.trim().split(/\s+/).includes(expected))
    throw new Error(`Tool version mismatch: expected ${expected}`);
}
export function validateTestOutput(output: string, expected: number) {
  const summaries = [
    ...output.matchAll(
      /test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out/g,
    ),
  ];
  if (
    summaries.length !== 1 ||
    Number(summaries[0][1]) !== expected ||
    summaries[0].slice(2).some((count) => Number(count) !== 0)
  )
    throw new Error('Missing or incomplete expected test execution');
  const ids = [...output.matchAll(/^test ([\w:]+) \.\.\. ok$/gm)].map((match) => match[1]);
  if (ids.length !== expected || new Set(ids).size !== expected)
    throw new Error('Missing or duplicate executed test identities');
  return ids;
}
export function validateAdvisories(output: string) {
  const rows: unknown[] = output
    .trim()
    .split('\n')
    .filter(Boolean)
    .map((line) => JSON.parse(line));
  const summaries = rows.filter((row) => isRecord(row) && row.type === 'summary');
  if (summaries.length !== 1) throw new Error('Missing advisory summary');
  const summary = summaries[0];
  if (!isRecord(summary) || !isRecord(summary.fields) || !isRecord(summary.fields.advisories))
    throw new Error('Malformed advisory summary');
  const advisories = summary.fields.advisories;
  if (advisories.errors !== 0 || advisories.warnings !== 0)
    throw new Error('Advisory errors or warnings require review');
  return advisories;
}
export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
export function requireUnchanged(before: unknown, after: unknown) {
  if (JSON.stringify(before) !== JSON.stringify(after))
    throw new Error('Source changed during lane execution');
}
