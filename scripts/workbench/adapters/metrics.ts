import type { Metric, Section, Source } from '../contracts/index.ts';
import { boundedRead } from '../files.ts';
import { isSource } from '../../quality/source-size.ts';
export async function metrics(
  root: string,
  paths: string[],
  source: Source,
): Promise<Section<Metric[]>> {
  const data: Metric[] = [];
  const result: Section<Metric[]> = { state: 'ready', diagnostics: [], data };
  try {
    const policy = JSON.parse((await boundedRead(root, 'quality/source-policy.json')).toString());
    const candidates = paths.filter((p) => isSource(p, '', policy) || p.startsWith('scripts/'));
    if (candidates.length > 10000) throw new Error('Source count bound');
    let bytes = 0;
    let maximum = 0;
    let violations = 0;
    for (const path of candidates) {
      const buffer = await boundedRead(root, path);
      bytes += buffer.length;
      if (bytes > 32 * 1024 * 1024) throw new Error('Source scan budget');
      const text = buffer.toString();
      if (!isSource(path, text, policy)) continue;
      const lines = text ? text.split('\n').length - Number(text.endsWith('\n')) : 0;
      maximum = Math.max(maximum, lines);
      if (lines > policy.maximumPhysicalLines) violations++;
    }
    for (const [id, label, value, unit] of [
      ['source-maximum', 'Largest handwritten source file', maximum, 'physical lines'],
      ['source-violations', 'Files exceeding source cap', violations, 'files'],
    ] as const)
      data.push({
        id,
        label,
        value,
        unit,
        status: 'measured',
        source,
        capturedAt: new Date().toISOString(),
        scope: 'Bounded current source scan; not execution of all quality gates',
        configuration: 'quality/source-policy.json',
        evidencePaths: ['quality/source-policy.json'],
      });
  } catch {
    result.state = 'partial';
    result.diagnostics.push({
      severity: 'error',
      code: 'source-metrics-unavailable',
      message: 'Source inventory missing, escaping or beyond bounds.',
    });
  }
  data.push({
    id: 'physical-latency',
    label: 'Physical pen-to-screen latency',
    value: null,
    unit: 'ms',
    status: 'missing',
    source: null,
    capturedAt: null,
    scope: 'Requires physical camera measurements',
    configuration: 'Not measured by this workbench',
    evidencePaths: [],
  });
  return result;
}
