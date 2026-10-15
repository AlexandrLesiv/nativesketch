import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { writeFileSync } from 'node:fs';

const revision = process.argv[2] ?? '712059d5c4470c9770d08c42de8f1391884a66be';
const output = process.argv[3];
/** @param {...string} args */
const git = (...args) => execFileSync('git', args, { maxBuffer: 16 * 1024 * 1024 });
const resolved = git('rev-parse', '--verify', `${revision}^{commit}`).toString().trim();
const files = git('ls-tree', '-rz', '--name-only', resolved).toString().split('\0').filter(Boolean);
const source = /\.(rs|swift|kt|kts|cs|cpp|h|ts|js|mjs|cjs|py|sh|ps1|html|css)$/;
/** @param {string} path */
const extraction = (path) => {
  if (path.startsWith('scripts/')) return '016';
  if (path.startsWith('crates/')) return '018';
  if (path.startsWith('apps/macos/')) return '019';
  if (path.startsWith('apps/windows/')) return '004';
  if (path.startsWith('apps/android/')) return '010';
  if (path.startsWith('apps/linux/')) return '011';
  if (path.startsWith('apps/web/')) return '012';
  return '009';
};
/** @param {string} path @param {string} content */
const classify = (path, content) => {
  if (path.endsWith('.lock') || path.endsWith('package-lock.json')) return 'lockfile';
  if (source.test(path) || (path.startsWith('scripts/') && content.startsWith('#!'))) {
    return 'handwritten-source';
  }
  if (path.includes('/fixtures/')) return 'fixture-data';
  if (path.endsWith('.md')) return 'documentation';
  return 'configuration';
};
const entries = files.map((path) => {
  const bytes = git('show', `${resolved}:${path}`);
  const text = bytes.toString();
  const lines = text.length === 0 ? 0 : text.split('\n').length - Number(text.endsWith('\n'));
  const category = classify(path, text);
  return {
    path,
    category,
    bytes: bytes.length,
    physicalLines: lines,
    sha256: createHash('sha256').update(bytes).digest('hex'),
    ...(category === 'handwritten-source' && lines > 200
      ? { extractionTask: extraction(path) }
      : {}),
  };
});
const categories = Object.fromEntries(
  [
    'handwritten-source',
    'generated',
    'vendor',
    'lockfile',
    'fixture-data',
    'documentation',
    'configuration',
  ].map((category) => {
    const matching = entries.filter((entry) => entry.category === category);
    return [
      category,
      { files: matching.length, physicalLines: matching.reduce((n, e) => n + e.physicalLines, 0) },
    ];
  }),
);
const report = {
  schemaVersion: 1,
  revision: resolved,
  policy:
    'All tracked files classified; physical lines include comments and blanks. No generated or vendor source in this revision.',
  categories,
  oversized: entries.filter((entry) => entry.extractionTask),
  entries,
};
const json = JSON.stringify(report, null, 2) + '\n';
if (output) writeFileSync(output, json);
else process.stdout.write(json);
