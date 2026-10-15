import { randomUUID } from 'node:crypto';
import { mkdir, readFile, readdir, writeFile, stat } from 'node:fs/promises';
import { resolve, dirname, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { discoverCargo, executeCargo } from './evidence/cargo.mjs';
import { executeNode } from './evidence/node.mjs';
import { sourceIdentity } from './evidence/identity.mjs';
import { buildReport, renderHtml } from './evidence/report.mjs';
import { validateDiscovery, validateExecution, requireValue } from './evidence/validate.mjs';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const out = resolve(root, 'artifacts/evidence');
const args = process.argv.slice(2);
const execute = args.includes('--run');
const gate = args.includes('--gate');
let attempt = 1;
/** @type {string[]} */
const inputs = [];
/** @type {string[]} */
const required = [];
/** @type {string[]} */
const nodeFiles = [];
for (let index = 0; index < args.length; index++) {
  if (args[index] === '--attempt') {
    attempt = Number(args[++index]);
    requireValue(Number.isInteger(attempt) && attempt > 0, 'Invalid attempt');
  } else if (args[index] === '--require')
    required.push(...(args[++index] ?? '').split(',').filter(Boolean));
  else if (args[index] === '--node') nodeFiles.push(resolve(root, args[++index] ?? ''));
  else if (args[index] === '--input') inputs.push(resolve(root, args[++index] ?? ''));
  else if (!['--run', '--gate'].includes(args[index]))
    throw new Error(`Unknown option: ${args[index]}`);
}
await mkdir(out, { recursive: true });
const source = await sourceIdentity(root);
const cargo = await discoverCargo(root);
/** @type {import('./evidence/types.mjs').Discovery} */
const discovery = { schemaVersion: 1, source, tests: cargo.tests };
/** @type {import('./evidence/types.mjs').Execution[]} */
const executions = [];
if (attempt > 1)
  executions.push(...JSON.parse(await readFile(resolve(out, 'results.json'), 'utf8')));
if (execute)
  executions.push({
    schemaVersion: 1,
    source,
    results: (await executeCargo(root, cargo)).map((result) => ({ ...result, attempt })),
  });
if (nodeFiles.length) {
  const node = await executeNode(root, nodeFiles);
  discovery.tests.push(...node.tests);
  executions.push({
    schemaVersion: 1,
    source,
    results: node.results.map((result) => ({ ...result, attempt })),
  });
}
for (const path of inputs) {
  const envelope = JSON.parse(await readFile(path, 'utf8'));
  if (envelope.tests) {
    validateDiscovery(envelope);
    requireValue(
      JSON.stringify(envelope.source) === JSON.stringify(source),
      'Imported discovery has stale source',
    );
    discovery.tests.push(...envelope.tests);
  } else {
    validateExecution(envelope);
    for (const result of envelope.results) {
      for (let i = 0; i < result.artifacts.length; i++) {
        const artifact = resolve(dirname(path), result.artifacts[i]);
        requireValue(
          (await stat(artifact)).isFile() || (await stat(artifact)).isDirectory(),
          'Missing artifact',
        );
        const link = relative(out, artifact).replaceAll('\\', '/');
        requireValue(
          !link.startsWith('..'),
          'Imported artifacts must be inside artifacts/evidence',
        );
        result.artifacts[i] = link;
      }
    }
    executions.push(envelope);
  }
}
const features = await Promise.all(
  (await readdir(resolve(root, 'quality/features')))
    .filter((name) => name.endsWith('.json'))
    .map(async (name) =>
      JSON.parse(await readFile(resolve(root, 'quality/features', name), 'utf8')),
    ),
);
requireValue(
  JSON.stringify(await sourceIdentity(root)) === JSON.stringify(source),
  'Source changed during evidence run',
);
const report = buildReport(features, discovery, executions);
await writeFile(resolve(out, 'discovery.json'), JSON.stringify(discovery, null, 2));
await writeFile(resolve(out, 'results.json'), JSON.stringify(executions, null, 2));
await mkdir(resolve(out, 'history'), { recursive: true });
await writeFile(
  resolve(out, 'history', `${Date.now()}-${randomUUID()}.json`),
  JSON.stringify({ discovery, executions }, null, 2),
  { flag: 'wx' },
);
await writeFile(resolve(out, 'coverage.json'), JSON.stringify(report, null, 2));
await writeFile(resolve(out, 'coverage.html'), renderHtml(report));
console.log(
  `${report.rows.length} acceptance/platform rows; ${report.rows.filter((row) => row.status === 'passed').length} passed. ${relative(root, out)}/coverage.html`,
);
for (const id of required)
  requireValue(
    report.rows.some((row) => row.caseId === id),
    `Unknown required case: ${id}`,
  );
if (
  report.issues.length ||
  (gate && !report.passed) ||
  report.rows.some((row) => required.includes(row.caseId) && row.status !== 'passed')
)
  process.exitCode = 1;
