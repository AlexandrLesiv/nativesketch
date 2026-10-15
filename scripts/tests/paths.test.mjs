import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import { isContained } from '../lib/paths.mjs';
await test('containment rejects cross-drive and parent escapes on Windows', () => {
  assert.equal(isContained('C:\\root', 'D:\\outside', path.win32), false);
  assert.equal(isContained('C:\\root', 'C:\\elsewhere', path.win32), false);
  assert.equal(isContained('C:\\root', 'C:\\root\\file', path.win32), true);
  assert.equal(isContained('/root', '/rooted/file', path.posix), false);
  assert.equal(isContained('/root', '/root/..visible', path.posix), true);
});
