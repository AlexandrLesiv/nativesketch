export interface JvmCase {
  name: string;
  className: string;
  status: 'passed' | 'failed' | 'skipped';
  durationMs: number;
  timestamp: string;
}
function attributes(text: string): Record<string, string> {
  return Object.fromEntries(
    [...text.matchAll(/([\w.-]+)="([^"]*)"/g)].map((match) => [
      match[1],
      (match[2] ?? '')
        .replaceAll('&quot;', '"')
        .replaceAll('&apos;', "'")
        .replaceAll('&lt;', '<')
        .replaceAll('&gt;', '>')
        .replaceAll('&amp;', '&'),
    ]),
  );
}
/** Gradle JUnit XML only; no DTD/entity execution and bounded input. Raw XML is retained. */
export function parseJvmXml(text: string): JvmCase[] {
  if (Buffer.byteLength(text) > 4_000_000 || /<!DOCTYPE|<!ENTITY/.test(text))
    throw new Error('Unsupported JUnit XML');
  const suite = /<testsuite\b([^>]*)>/.exec(text);
  if (!suite || !text.includes('</testsuite>')) throw new Error('Missing JUnit suite');
  const summary = attributes(suite[1]);
  const rows = [...text.matchAll(/<testcase\b([^>]*?)(?:\/>|>([\s\S]*?)<\/testcase>)/g)].map(
    (match): JvmCase => {
      const attr = attributes(match[1]);
      const body = match[2] ?? '';
      const durationMs = Number(attr.time) * 1000;
      if (!attr.name || !attr.classname || !Number.isFinite(durationMs) || durationMs < 0)
        throw new Error('Invalid JUnit case');
      return {
        name: attr.name,
        className: attr.classname,
        durationMs,
        timestamp: `${summary.timestamp}Z`.replace('ZZ', 'Z'),
        status: /<(failure|error)\b/.test(body)
          ? 'failed'
          : /<skipped\b/.test(body)
            ? 'skipped'
            : 'passed',
      };
    },
  );
  if (
    !rows.length ||
    Number(summary.tests) !== rows.length ||
    rows.some((row) => !Number.isFinite(Date.parse(row.timestamp)))
  )
    throw new Error('Empty or inconsistent JUnit enumeration');
  if (new Set(rows.map((row) => row.className + ':' + row.name)).size !== rows.length)
    throw new Error('Duplicate JUnit case');
  const failed = rows.filter((row) => row.status === 'failed').length;
  if (
    failed !== Number(summary.failures) + Number(summary.errors) ||
    rows.filter((row) => row.status === 'skipped').length !== Number(summary.skipped)
  )
    throw new Error('JUnit counters disagree with cases');
  return rows;
}
export function requireLane(rows: JvmCase[], classes: string[], discovered: string[]): void {
  const actual = new Set(rows.map((row) => row.className.split('.').at(-1)));
  if (actual.size !== classes.length || classes.some((name) => !actual.has(name)))
    throw new Error('Missing or unexpected JVM test class');
  const actualIds = rows.map((row) => row.className + ':' + row.name).sort();
  if (JSON.stringify(actualIds) !== JSON.stringify([...discovered].sort()))
    throw new Error('Executed methods differ from compiled discovery');
  if (rows.some((row) => row.status !== 'passed'))
    throw new Error('JVM execution includes failure or skip');
}
