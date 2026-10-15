import assert from 'node:assert/strict';
import { test } from 'node:test';
import { checkDependencies } from './dependencies.mjs';

const policy = { packages: { 'sketch-core': { allowedNormalDependencies: [], allowedBuildDependencies: [] } } };

test('rejects a renderer dependency even when Cargo records a renamed package', () => {
  const metadata = { packages: [{ name: 'sketch-core', dependencies: [
    { name: 'skia-safe', rename: 'drawing', kind: null },
  ] }] };
  assert.deepEqual(checkDependencies(metadata, policy), ['sketch-core -> skia-safe: unapproved normal dependency']);
});

test('rejects platform-specific and build-time dependencies, permits dev-only tests', () => {
  const metadata = { packages: [{ name: 'sketch-core', dependencies: [
    { name: 'windows', kind: null, target: 'cfg(windows)' },
    { name: 'bindgen', kind: 'build' },
    { name: 'proptest', kind: 'dev' },
  ] }] };
  assert.equal(checkDependencies(metadata, policy).length, 2);
});

test('missing package cannot silently disable a dependency rule', () => {
  assert.deepEqual(checkDependencies({ packages: [] }, policy), ['Missing required package: sketch-core']);
});
