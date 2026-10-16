export type Package = { name: string; dependencies: { name: string; kind: string | null }[] };
export type DependencyPolicy = {
  packages: Record<
    string,
    { allowedNormalDependencies: string[]; allowedBuildDependencies: string[] }
  >;
};

/** Inspect Cargo's parsed dependency declarations, including renamed/target-specific
  dependencies. An explicit small allowlist prevents unknown platform libraries
  from bypassing a blacklist. Cargo itself validates the package graph. */
export function checkDependencies(
  metadata: {
    packages: Package[];
  },
  policy: DependencyPolicy,
) {
  const failures: string[] = [];
  for (const [name, allowed] of Object.entries(policy.packages)) {
    const pkg = metadata.packages.find((entry) => entry.name === name);
    if (!pkg) {
      failures.push(`Missing required package: ${name}`);
      continue;
    }
    for (const dependency of pkg.dependencies) {
      if (dependency.kind === 'dev') continue;
      const permitted =
        dependency.kind === 'build'
          ? allowed.allowedBuildDependencies
          : allowed.allowedNormalDependencies;
      if (!permitted.includes(dependency.name)) {
        failures.push(
          `${name} -> ${dependency.name}: unapproved ${dependency.kind ?? 'normal'} dependency`,
        );
      }
    }
  }
  return failures;
}
