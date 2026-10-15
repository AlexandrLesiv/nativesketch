import test from 'node:test';
import assert from 'node:assert/strict';
import { cargoTestPlatform } from '../evidence/cargo-platform.ts';

await test('portable Cargo contracts retain their scope without claiming native effects', () => {
  for (const host of ['darwin', 'win32', 'linux', 'unknown']) {
    assert.equal(cargoTestPlatform(undefined, host), 'portable');
    assert.equal(cargoTestPlatform('portable', host), 'portable');
  }
});

await test('native Cargo effects are assigned only to the actual execution host', () => {
  for (const [host, expected] of [
    ['darwin', 'macos'],
    ['win32', 'windows'],
    ['linux', 'linux'],
  ]) {
    assert.equal(cargoTestPlatform('host', host), expected);
  }
});

await test('Cargo scope rejects typos, arbitrary platform claims and unsupported hosts', () => {
  for (const declaration of [null, false, {}, [], 'macos', 'windows', 'hots']) {
    assert.throws(() => cargoTestPlatform(declaration, 'darwin'), /Unknown Cargo/);
  }
  assert.throws(() => cargoTestPlatform('host', 'unknown'), /Unsupported native Cargo/);
});
