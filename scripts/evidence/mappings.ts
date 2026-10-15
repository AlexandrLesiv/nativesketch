import type { DiscoveredTest, Discovery, Feature, Requirement } from './types.ts';

export function matchesRequirement(test: DiscoveredTest, requirement: Requirement) {
  return (
    test.platform === requirement.platform &&
    test.level === requirement.level &&
    (requirement.pointerWidth === undefined || test.pointerWidth === requirement.pointerWidth)
  );
}

export function wasEnumerated(id: string, requirement: Requirement, discovery: Discovery) {
  const runner = id.split(':')[0];
  return [...(discovery.scopes ?? []), ...discovery.tests].some(
    (scope) =>
      scope.runner === runner &&
      scope.platform === requirement.platform &&
      scope.level === requirement.level,
  );
}

export function mappingIssues(features: Feature[], discovery: Discovery) {
  const tests = new Map(discovery.tests.map((test) => [test.id, test]));
  const issues: string[] = [];
  for (const item of features.flatMap((feature) => feature.cases)) {
    for (const id of item.testIds) {
      const test = tests.get(id);
      if (!test) issues.push(`Undiscovered required test: ${id}`);
      else if (!item.requirements.some((r) => !r.coverage && matchesRequirement(test, r)))
        issues.push(`Orphan mapping: ${item.id} -> ${id}`);
    }
    for (const requirement of item.requirements) {
      for (const id of requirement.coverage?.testIds ?? []) {
        const test = tests.get(id);
        if (!test && wasEnumerated(id, requirement, discovery))
          issues.push(`Undiscovered required test: ${id}`);
        else if (test && !matchesRequirement(test, requirement))
          issues.push(`Orphan mapping: ${item.id} -> ${id}`);
      }
    }
  }
  return issues;
}
