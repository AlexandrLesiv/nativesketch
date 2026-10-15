import test from 'node:test';
import assert from 'node:assert/strict';
import { e2eOptions } from '../e2e/options.ts';

const platform = ['--platform', 'macos'];
await test('native E2E defaults to background preparation and preserves prepare-only alias', () => {
  assert.equal(e2eOptions(platform).execute, false);
  assert.equal(e2eOptions([...platform, '--prepare-only']).execute, false);
});
await test('foreground execution requires explicit interactive flag', () => {
  assert.equal(e2eOptions([...platform, '--interactive']).execute, true);
  assert.throws(
    () => e2eOptions([...platform, '--interactive', '--prepare-only']),
    /mutually exclusive/,
  );
});
await test('CLI validates retry identity and retains explicit output', () => {
  assert.deepEqual(e2eOptions([...platform, '--attempt', '2', '--output', 'artifacts/retry']), {
    execute: false,
    attempt: 2,
    output: 'artifacts/retry',
  });
  assert.throws(() => e2eOptions([...platform, '--attempt', '0']), /Usage/);
  assert.throws(() => e2eOptions([...platform, '--attempt', '1.5']), /Usage/);
  assert.throws(() => e2eOptions([...platform, '--foreground']), /Unknown option/);
});
