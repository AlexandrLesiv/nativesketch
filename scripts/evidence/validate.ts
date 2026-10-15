import type { Source, Discovery, Execution, Feature } from './types.ts';
import { loadSchema, validateSchema } from './schema.ts';
const schemas = {
  feature: loadSchema('feature'),
  discovery: loadSchema('discovery'),
  execution: loadSchema('execution'),
};
const hash = /^[a-f0-9]{64}$/;

export function requireValue(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

export function unique(values: string[], label: string) {
  requireValue(new Set(values).size === values.length, `Duplicate ${label}`);
}

export function validateSource(source: Source) {
  requireValue(/^[a-f0-9]{40}$/.test(source.revision), 'Invalid source revision');
  requireValue(
    source.dirtyDiffSha256 === null || hash.test(source.dirtyDiffSha256),
    'Invalid dirty diff hash',
  );
}

export function validateDiscovery(discovery: Discovery) {
  validateSchema(discovery, schemas.discovery);
  requireValue(discovery.schemaVersion === 1, 'Unknown discovery schema');
  validateSource(discovery.source);
  unique(
    discovery.tests.map((test) => test.id),
    'discovery ID',
  );
  for (const test of discovery.tests) {
    requireValue(
      test.testBinarySha256 === undefined || hash.test(test.testBinarySha256),
      'Invalid test binary hash',
    );
    requireValue(
      test.runner !== 'xctest' || test.testBinarySha256,
      'Missing XCTest binary identity',
    );
    requireValue(test.id.startsWith(`${test.runner}:`), `Invalid namespace ${test.id}`);
    requireValue(
      test.platform && test.level && hash.test(test.binarySha256),
      `Invalid discovery ${test.id}`,
    );
    requireValue(
      test.pointerWidth === undefined || [32, 64].includes(test.pointerWidth),
      'Invalid pointer width',
    );
  }
}

export function validateExecution(execution: Execution) {
  validateSchema(execution, schemas.execution);
  requireValue(execution.schemaVersion === 1, 'Unknown execution schema');
  validateSource(execution.source);
  unique(
    execution.results.map((r) => `${r.testId}:${r.attempt}`),
    'result attempt',
  );
  for (const r of execution.results) {
    requireValue(
      r.testBinarySha256 === undefined || hash.test(r.testBinarySha256),
      'Invalid test binary hash',
    );
    requireValue(
      ['passed', 'failed', 'skipped', 'blocked', 'unsupported'].includes(r.status),
      'Unknown result status',
    );
    requireValue(Number.isInteger(r.attempt) && r.attempt > 0, 'Invalid attempt');
    requireValue(
      Number.isFinite(r.durationMs) &&
        r.durationMs >= 0 &&
        Number.isFinite(Date.parse(r.timestamp)),
      'Invalid timing',
    );
    requireValue(
      hash.test(r.binarySha256) && r.platform && r.toolchain && r.device,
      'Missing execution identity',
    );
    requireValue(
      typeof r.environment.valid === 'boolean' && Array.isArray(r.environment.reasons),
      'Invalid environment',
    );
    requireValue(
      r.environment.valid || r.environment.reasons.length > 0,
      'Invalid environment needs reason',
    );
    for (const artifact of r.artifacts) {
      requireValue(
        artifact.length > 0 &&
          !artifact.startsWith('/') &&
          !artifact.includes('\\') &&
          !artifact.split('/').includes('..') &&
          !artifact.includes(':'),
        'Unsafe artifact path',
      );
    }
  }
}

export function validateFeatures(features: Feature[]) {
  unique(
    features.map((feature) => feature.id),
    'feature ID',
  );
  unique(
    features.flatMap((feature) => feature.cases.map((item) => item.id)),
    'case ID',
  );
  for (const feature of features) {
    validateSchema(feature, schemas.feature);
    requireValue(
      feature.schemaVersion === 1 && feature.id && feature.description,
      'Invalid feature',
    );
    for (const item of feature.cases) {
      requireValue(
        item.id && item.expected && /^\d{3}$/.test(item.ownerTask),
        'Invalid acceptance case',
      );
      requireValue(
        ['planned', 'partial', 'implemented', 'unsupported', 'not-applicable'].includes(
          item.automation,
        ),
        'Unknown automation state',
      );
      unique(item.testIds, 'case test mapping');
      requireValue(item.requirements.length > 0, 'Missing case requirements');
      unique(
        item.requirements.map((r) => `${r.platform}:${r.level}:${r.pointerWidth ?? ''}`),
        'requirement',
      );
      for (const r of item.requirements) requireValue(r.platform && r.level, 'Invalid requirement');
      for (const gap of item.gaps)
        requireValue(gap.reason && /^\d{3}$/.test(gap.ownerTask), 'Invalid gap');
      if (item.automation === 'implemented')
        requireValue(item.testIds.length, 'Implemented case needs tests');
    }
  }
}
