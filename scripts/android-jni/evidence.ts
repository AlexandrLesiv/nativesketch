import { readdir, readFile, cp, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import type { Source, Discovery, Execution } from '../evidence/types.ts';
import { validateDiscovery, validateExecution } from '../evidence/validate.ts';
import { parseJvmXml, requireLane } from './xml.ts';
import { expectedClasses } from './contracts.ts';
export async function harvest(root: string, output: string, discovered: string[]) {
  const lanes = [];
  const issues: string[] = [];
  for (const [name, classes] of Object.entries(expectedClasses)) {
    const rows = [];
    try {
      const path = join(root, 'apps/android/app/build/test-results', name);
      await cp(path, join(output, name), { recursive: true });
      for (const file of (await readdir(path)).filter((item) => item.endsWith('.xml')))
        rows.push(...parseJvmXml(await readFile(join(path, file), 'utf8')));
      requireLane(
        rows,
        classes,
        discovered.filter((id) =>
          classes.some((name) => id.startsWith('dev.nativesketch.portable.' + name + ':')),
        ),
      );
    } catch (error) {
      issues.push(`${name}: ${String(error)}`);
    }
    lanes.push({ name, rows });
  }
  return { lanes, issues };
}
export function normalizedRecords(
  data: Awaited<ReturnType<typeof harvest>>,
  source: Source,
  binaries: { production: string; probe: string; classes: string | null },
  jvm: string,
  issues: string[],
) {
  const platform = process.platform === 'darwin' ? 'macos' : process.platform;
  const tests: Discovery['tests'] = [],
    results: Execution['results'] = [];
  for (const lane of data.lanes)
    for (const row of lane.rows) {
      const binarySha256 =
        lane.name === 'portableJniPendingExceptionTest' ? binaries.probe : binaries.production;
      const id = `jvm-jni:${platform}:${row.className}:${row.name}`;
      tests.push({
        id,
        runner: 'jvm-jni',
        level: 'bridge',
        platform,
        binarySha256,
        ...(binaries.classes ? { testBinarySha256: binaries.classes } : {}),
      });
      results.push({
        testId: id,
        attempt: 1,
        status: row.status,
        durationMs: row.durationMs,
        timestamp: row.timestamp,
        binarySha256,
        ...(binaries.classes ? { testBinarySha256: binaries.classes } : {}),
        platform,
        toolchain: jvm,
        device: `host-jvm-${process.arch}`,
        environment: { valid: issues.length === 0, reasons: issues },
        artifacts: ['identity.json', 'gradle.log', lane.name],
      });
    }
  const discovery: Discovery = { schemaVersion: 1, source, tests };
  const execution: Execution = { schemaVersion: 1, source, results };
  validateDiscovery(discovery);
  validateExecution(execution);
  return { discovery, execution };
}
export async function normalized(
  output: string,
  ...args: Parameters<typeof normalizedRecords>
): Promise<void> {
  const records = normalizedRecords(...args);
  await writeFile(join(output, 'discovery.json'), JSON.stringify(records.discovery, null, 2));
  await writeFile(join(output, 'execution.json'), JSON.stringify(records.execution, null, 2));
}
