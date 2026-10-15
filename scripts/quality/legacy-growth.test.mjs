import assert from 'node:assert/strict';
import { test } from 'node:test';
import { checkLegacyGrowth } from './legacy-growth.mjs';

test('a file cannot regrow after shrinking even below the original debt ceiling', () => {
  const changes = [
    { path: 'legacy.rs', before: 210, after: 201, revision: 'shrink' },
    { path: 'legacy.rs', before: 201, after: 205, revision: 'regrow' },
  ];
  assert.deepEqual(checkLegacyGrowth(changes, 200),
    ['legacy.rs: oversized source grew 201 -> 205 at regrow']);
});

test('completed extraction permits ordinary growth only within the final limit', () => {
  const changes = [
    { path: 'legacy.rs', before: 210, after: 100, revision: 'extract' },
    { path: 'legacy.rs', before: 100, after: 150, revision: 'small-change' },
    { path: 'legacy.rs', before: 150, after: 201, revision: 'over-limit' },
  ];
  assert.equal(checkLegacyGrowth(changes, 200).length, 1);
});

test('repairing a historical growth violation clears the failure without rewriting history', () => {
  const changes = [
    { path: 'legacy.rs', before: 210, after: 201, revision: 'shrink' },
    { path: 'legacy.rs', before: 201, after: 205, revision: 'regrow' },
    { path: 'legacy.rs', before: 205, after: 201, revision: 'repair' },
  ];
  assert.deepEqual(checkLegacyGrowth(changes, 200), []);
});
