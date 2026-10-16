import assert from 'node:assert/strict';
import { test } from 'node:test';
import { checkNativeBoundary } from './native-boundary.ts';

const header = 'crates/sketch-native-ffi/include/';
const modulePath = 'apps/macos/Sources/CSketchSession/module.modulemap';
const app = 'apps/macos/Sources/NativeSketch/';
const manifest = 'apps/macos/Package.swift';
function fixture() {
  return new Map([
    [manifest, 'dependencies: ["CSketchSession"] // libsketch_native_ffi.a'],
    [
      modulePath,
      'module CSketchSession [system] { header "../../../../crates/sketch-native-ffi/include/sketch_session_v1.h" export * }',
    ],
    [header + 'sketch_session_v1.h', '#include "sketch_values_v1.h"\n'],
    [header + 'sketch_values_v1.h', '#include <stdint.h>\n'],
    [app + 'Consumer.swift', 'import CSketchSession\n'],
  ]);
}

await test('accepts versioned umbrella with shared transitive headers and cycles', () => {
  const files = fixture();
  files.set(header + 'sketch_values_v1.h', '#include <stdint.h>\n#include "sketch_session_v1.h"');
  assert.deepEqual(checkNativeBoundary(files), []);
});

await test('rejects legacy imports, scoped imports and manual symbol declarations', () => {
  for (const code of [
    'import CSketchFFI',
    '@preconcurrency import func CSketchFFI.sketch_new',
    '@_silgen_name("sketch_new") func old()',
  ]) {
    const files = fixture();
    files.set(app + 'Consumer.swift', code);
    assert.equal(checkNativeBoundary(files).length, 1, code);
  }
});

await test('rejects transitive legacy and test headers, unknown includes and missing headers', () => {
  for (const include of [
    '../../sketch-ffi/include/sketch_ffi.h',
    'sketch_test_v1.h',
    'missing.h',
  ]) {
    const files = fixture();
    files.set(header + 'sketch_values_v1.h', `#include "${include}"`);
    assert.equal(checkNativeBoundary(files).length, 1, include);
  }
  for (const directive of ['#include BRIDGE', '#import <legacy.h>']) {
    const files = fixture();
    files.set(header + 'sketch_values_v1.h', directive);
    assert.equal(checkNativeBoundary(files).length, 1, directive);
  }
});

await test('rejects old package linkage, extra module dependencies and removed umbrella', () => {
  for (const dependency of ['CSketchFFI', 'libsketch_ffi.a', 'sketch_ffi']) {
    const files = fixture();
    files.set(manifest, files.get(manifest) + ` "${dependency}"`);
    assert.equal(checkNativeBoundary(files).length, 1, dependency);
  }
  const files = fixture();
  files.set(modulePath, files.get(modulePath) + '\nuse CSketchFFI');
  assert.equal(checkNativeBoundary(files).length, 1);
  files.delete(modulePath);
  assert.equal(checkNativeBoundary(files).length, 1);
});
