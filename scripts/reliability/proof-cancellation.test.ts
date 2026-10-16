import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { sanitizerProof } from './asan-proof.ts';
import { recorded } from './evidence.ts';

await test('pre-cancelled sanitizer proof cannot inspect artifacts or start a process', async () => {
  const signal = AbortSignal.abort(new Error('cancelled proof'));
  await assert.rejects(
    sanitizerProof('/missing', '/missing', '/missing', '', '', '', signal),
    /cancelled proof/,
  );
});

await test('cancelling during sanitizer proof stops its command and cannot publish proof', async () => {
  const root = await mkdtemp(join(tmpdir(), 'sketch-proof-cancel-'));
  const output = join(root, 'evidence');
  const cache = join(root, 'target/reliability-asan');
  await mkdir(cache, { recursive: true });
  await mkdir(output);
  const binary = join(cache, 'tests');
  await writeFile(binary, 'disposable artifact fixture');
  const log = JSON.stringify({
    reason: 'compiler-artifact',
    target: { name: 'sketch_ffi' },
    profile: { test: true },
    executable: binary,
  });
  const abort = new AbortController();
  const steps: string[] = [];
  const execute: typeof recorded = async (directory, name, _executable, _args, options) => {
    steps.push(name);
    if (name === 'asan-symbols') return 'T ___asan_init\nT ___asan_report_load1\n';
    const timer = setTimeout(() => abort.abort(), 100);
    try {
      return await recorded(
        directory,
        name,
        process.execPath,
        ['-e', 'setInterval(() => {}, 1000)'],
        { ...options, capture: true, timeout: 2000 },
      );
    } finally {
      clearTimeout(timer);
    }
  };
  try {
    await assert.rejects(
      sanitizerProof(root, output, root, 'fixture', 'fixture', log, abort.signal, execute),
      /Process cancelled/,
    );
    assert.deepEqual(steps, ['asan-symbols', 'canary-build']);
    assert.equal((await readdir(output)).includes('instrumentation.json'), false);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
