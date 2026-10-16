import { test } from 'node:test';
import assert from 'node:assert/strict';
import { workbenchOptions } from '../lib/workbench-options.ts';
await test('workbench CLI separates loopback service from explicit new artifact destinations', () => {
  assert.deepEqual(workbenchOptions(['serve'], '/project'), { mode: 'serve', port: 0 });
  assert.deepEqual(workbenchOptions(['serve', '--port', '8079'], '/project'), {
    mode: 'serve',
    port: 8079,
  });
  assert.deepEqual(
    workbenchOptions(['export', '--output', 'artifacts/My report.html'], '/project'),
    {
      mode: 'export',
      output: '/project/artifacts/My report.html',
    },
  );
  assert.deepEqual(
    workbenchOptions(['snapshot', '--output', 'artifacts/report.json'], '/project'),
    {
      mode: 'snapshot',
      output: '/project/artifacts/report.json',
    },
  );
});
await test('workbench CLI rejects ambiguous, unsupported and foreground options', () => {
  for (const args of [
    [],
    ['run'],
    ['serve', '--interactive'],
    ['serve', '--port', '-1'],
    ['serve', '--port', '1.5'],
    ['serve', '--port', '65536'],
    ['serve', '--port', 'Infinity'],
    ['serve', '--port', '80', '--port', '90'],
    ['serve', '--output', 'file'],
    ['snapshot'],
    ['export', '--output'],
    ['export', '--output', '--port'],
    ['export', '--output', 'file', '--port', '80'],
    ['export', '--shell', 'command'],
  ])
    assert.throws(() => workbenchOptions(args, '/project'), JSON.stringify(args));
});
