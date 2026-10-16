import test from 'node:test';
import assert from 'node:assert/strict';
import { ProcessFailure } from '../lib/process-failure.ts';
import {
  sanitizerEnvironment,
  instrumentedArtifact,
  requireInstrumentation,
  requireCanaryFailure,
} from './asan-proof.ts';

await test('sanitizer uses encoded flags rather than a shadowed RUSTFLAGS value', () => {
  const env = sanitizerEnvironment();
  assert.equal(env.CARGO_ENCODED_RUSTFLAGS.split('\x1f').includes('-Zsanitizer=address'), true);
  assert.equal(env.RUSTFLAGS, '');
});
await test('sanitizer refuses missing artifacts and uninstrumented passing binaries', () => {
  assert.throws(() => instrumentedArtifact('test result: ok. 6 passed'), /Missing/);
  const artifact = JSON.stringify({
    reason: 'compiler-artifact',
    target: { name: 'sketch_ffi' },
    profile: { test: true },
    executable: '/fixture/tests',
  });
  assert.equal(instrumentedArtifact(artifact), '/fixture/tests');
  assert.throws(() => instrumentedArtifact(`${artifact}\n${artifact}`), /ambiguous/);
  assert.throws(() => requireInstrumentation('T _main\n'), /lacks/);
  assert.throws(() => requireInstrumentation('T ___asan_init\n'), /lacks/);
  assert.doesNotThrow(() => requireInstrumentation('T ___asan_init\nT ___asan_report_load1\n'));
});
await test('sanitizer canary requires its exact detected fault and expected exit', () => {
  const text = 'ERROR: AddressSanitizer: heap-use-after-free\nREAD of size 1';
  assert.doesNotThrow(() => requireCanaryFailure(new ProcessFailure('canary', 86, null, '', text)));
  for (const error of [
    undefined,
    new Error(text),
    new ProcessFailure('canary', 1, null, '', text),
    new ProcessFailure('canary', 86, null, '', 'assertion failed'),
    new ProcessFailure('canary', 86, null, 'Process timed out', text),
    new ProcessFailure('canary', 86, 'SIGABRT', '', text),
  ])
    assert.throws(() => requireCanaryFailure(error));
});
