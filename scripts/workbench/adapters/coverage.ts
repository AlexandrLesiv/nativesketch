import type { Source, Section, CoverageData } from '../contracts/index.ts';
import type { Discovery, Execution, Feature } from '../../evidence/types.ts';
import { buildReport } from '../../evidence/report.ts';
import { validateFeatures, validateDiscovery, validateExecution } from '../../evidence/validate.ts';
import { boundedRead, sanitize } from '../files.ts';
export async function coverage(
  root: string,
  paths: string[],
  source: Source,
): Promise<Section<CoverageData>> {
  const data: CoverageData = {
    rows: [],
    issues: [],
    discoverySource: null,
    freshness: 'unknown',
    evidencePaths: [],
  };
  const result: Section<CoverageData> = { state: 'ready', diagnostics: [], data };
  try {
    const features: Feature[] = [];
    const manifests = paths.filter((p) => /^quality\/features\/[^/]+\.json$/.test(p));
    if (manifests.length > 1000) throw new Error('Feature manifest count bound');
    let bytes = 0;
    let rows = 0;
    for (const path of manifests) {
      const buffer = await boundedRead(root, path);
      bytes += buffer.length;
      if (bytes > 12 * 1024 * 1024) throw new Error('Feature byte bound');
      const feature: Feature = JSON.parse(buffer.toString());
      validateFeatures([feature]);
      for (const item of feature.cases) rows += item.requirements.length;
      if (rows > 10000) throw new Error('Feature requirement row bound');
      features.push(feature);
    }
    validateFeatures(features);
    let discovery: Discovery = { schemaVersion: 1, source, tests: [] };
    let executions: Execution[] = [];
    try {
      discovery = JSON.parse(
        (await boundedRead(root, 'artifacts/evidence/discovery.json', 8 * 1024 * 1024)).toString(),
      );
      validateDiscovery(discovery);
      const envelope: Execution | Execution[] = JSON.parse(
        (await boundedRead(root, 'artifacts/evidence/results.json', 8 * 1024 * 1024)).toString(),
      );
      executions = Array.isArray(envelope) ? envelope : [envelope];
      for (const execution of executions) validateExecution(execution);
      data.discoverySource = discovery.source;
      data.freshness =
        JSON.stringify(source) === JSON.stringify(discovery.source) ? 'current' : 'stale';
      data.evidencePaths = ['artifacts/evidence/discovery.json', 'artifacts/evidence/results.json'];
    } catch (error) {
      if (!(error instanceof Error) || !('code' in error) || error.code !== 'ENOENT') throw error;
      result.state = 'missing';
      result.diagnostics.push({
        severity: 'warning',
        code: 'coverage-evidence-missing',
        message:
          'Current discovery/results pair unavailable. Registered requirements remain unexecuted.',
      });
      discovery = { schemaVersion: 1, source, tests: [] };
      executions = [];
    }
    const report = buildReport(features, discovery, executions);
    data.issues = result.state === 'missing' ? [] : report.issues;
    data.rows = report.rows.map((row) => {
      const item = features
        .find((f) => f.id === row.feature)
        ?.cases.find((c) => c.id === row.caseId);
      const { pointerWidth, ...fields } = row;
      return {
        ...fields,
        ...(pointerWidth === undefined ? {} : { pointerWidth }),
        ownerTask: item?.ownerTask ?? '',
        invariants: item?.invariants ?? [],
      };
    });
    if (data.issues.length) result.state = 'partial';
  } catch {
    result.state = 'invalid';
    result.diagnostics.push({
      severity: 'error',
      code: 'coverage-invalid',
      message: 'Coverage validation failed; no passing acceptance can be inferred.',
    });
  }
  return sanitize(result);
}
