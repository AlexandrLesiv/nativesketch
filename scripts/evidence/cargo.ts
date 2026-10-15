import type { DiscoveredTest, Result } from './types.ts';
import { performance } from 'node:perf_hooks';
import { run } from '../lib/process.ts';
import { fileHash } from './identity.ts';
import { cargoTestPlatform } from './cargo-platform.ts';
type CargoPackage = {
  id: string;
  name: string;
  metadata?: { 'native-sketch'?: { 'test-platform'?: unknown } };
};
/** Compile and interrogate real libtest binaries; no source matching. */
export async function discoverCargo(root: string) {
  const metadata = JSON.parse(
    await run('cargo', ['metadata', '--locked', '--no-deps', '--format-version=1'], {
      cwd: root,
      capture: true,
    }),
  );
  const packages = new Map<string, { name: string; platform: string }>(
    metadata.packages.map((item: CargoPackage) => [
      item.id,
      {
        name: item.name,
        platform: cargoTestPlatform(
          item.metadata?.['native-sketch']?.['test-platform'],
          process.platform,
        ),
      },
    ]),
  );
  const output = await run(
    'cargo',
    ['test', '--workspace', '--locked', '--no-run', '--message-format=json'],
    { cwd: root, capture: true },
  );
  const toolchain = await run('rustc', ['-Vv'], { cwd: root, capture: true });

  const tests: (DiscoveredTest & {
    executable: string;
    test: string;
  })[] = [];
  for (const line of output.split('\n')) {
    if (!line.startsWith('{')) continue;
    const record = JSON.parse(line);
    if (record.reason !== 'compiler-artifact' || !record.profile?.test || !record.executable)
      continue;
    const pkg = packages.get(record.package_id);
    if (!pkg) throw new Error(`Unknown Cargo package ${record.package_id}`);
    const target = record.target.kind.some((kind: string) => kind.endsWith('lib'))
      ? 'lib'
      : record.target.name;
    const listing = await run(record.executable, ['--list'], { cwd: root, capture: true });
    const binarySha256 = await fileHash(record.executable);
    for (const entry of listing.split('\n')) {
      if (!entry.endsWith(': test')) continue;
      const test = entry.slice(0, -6);
      tests.push({
        id: `cargo:${pkg.name}:${target}:${test}`,
        runner: 'cargo',
        level: target === 'lib' ? 'unit' : 'integration',
        platform: pkg.platform,
        pointerWidth: process.arch.includes('64') ? 64 : 32,
        binarySha256,
        executable: record.executable,
        test,
      });
    }
  }
  return { tests, toolchain };
}

export async function executeCargo(
  root: string,
  discovery: Awaited<ReturnType<typeof discoverCargo>>,
) {
  const results: Result[] = [];
  for (const test of discovery.tests) {
    const start = performance.now();
    let status = 'passed';
    let output = '';
    try {
      output = await run(test.executable, ['--exact', test.test, '--nocapture'], {
        cwd: root,
        capture: true,
      });
      if (!/test result: ok\. 1 passed; 0 failed; 0 ignored;/.test(output)) status = 'skipped';
    } catch (error) {
      output = String(error);
      status = 'failed';
    }
    results.push({
      testId: test.id,
      attempt: 1,
      status,
      durationMs: performance.now() - start,
      timestamp: new Date().toISOString(),
      binarySha256: test.binarySha256,
      platform: test.platform,
      toolchain: discovery.toolchain,
      device: `${process.platform}/${process.arch}`,
      environment: { valid: true, reasons: [] },
      artifacts: [],
      output,
    });
  }
  return results;
}
