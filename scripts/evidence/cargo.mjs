import { performance } from 'node:perf_hooks';
import { run } from '../lib/process.mjs';
import { fileHash } from './identity.mjs';
/** Compile and interrogate real libtest binaries; no source matching. @param {string} root */
export async function discoverCargo(root) {
  const metadata = JSON.parse(
    await run('cargo', ['metadata', '--locked', '--no-deps', '--format-version=1'], {
      cwd: root,
      capture: true,
    }),
  );
  const packages = new Map(
    metadata.packages.map((/** @type {{id:string,name:string}} */ item) => [item.id, item.name]),
  );
  const output = await run(
    'cargo',
    ['test', '--workspace', '--locked', '--no-run', '--message-format=json'],
    { cwd: root, capture: true },
  );
  const toolchain = await run('rustc', ['-Vv'], { cwd: root, capture: true });
  /** @type {(import('./types.mjs').DiscoveredTest & {executable:string,test:string})[]} */
  const tests = [];
  for (const line of output.split('\n')) {
    if (!line.startsWith('{')) continue;
    const record = JSON.parse(line);
    if (record.reason !== 'compiler-artifact' || !record.profile?.test || !record.executable)
      continue;
    const packageName = packages.get(record.package_id);
    if (!packageName) throw new Error(`Unknown Cargo package ${record.package_id}`);
    const target = record.target.kind.some((/** @type {string} */ kind) => kind.endsWith('lib'))
      ? 'lib'
      : record.target.name;
    const listing = await run(record.executable, ['--list'], { cwd: root, capture: true });
    const binarySha256 = await fileHash(record.executable);
    for (const entry of listing.split('\n')) {
      if (!entry.endsWith(': test')) continue;
      const test = entry.slice(0, -6);
      tests.push({
        id: `cargo:${packageName}:${target}:${test}`,
        runner: 'cargo',
        level: target === 'lib' ? 'unit' : 'integration',
        platform: 'portable',
        pointerWidth: process.arch.includes('64') ? 64 : 32,
        binarySha256,
        executable: record.executable,
        test,
      });
    }
  }
  return { tests, toolchain };
}
/** @param {string} root @param {Awaited<ReturnType<typeof discoverCargo>>} discovery */
export async function executeCargo(root, discovery) {
  /** @type {import('./types.mjs').Result[]} */
  const results = [];
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
      platform: 'portable',
      toolchain: discovery.toolchain,
      device: `${process.platform}/${process.arch}`,
      environment: { valid: true, reasons: [] },
      artifacts: [],
      output,
    });
  }
  return results;
}
