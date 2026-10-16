import { test } from 'node:test';
import assert from 'node:assert/strict';
import { descriptors, validateDescriptors, validateExports } from './contracts.ts';
import { parseJvmXml, requireLane } from './xml.ts';
const suite = (body: string, counts = 'tests="1" skipped="0" failures="0" errors="0"') =>
  `<testsuite ${counts} timestamp="2026-10-16T00:00:00">${body}</testsuite>`;
const caseXml = '<testcase name="method" classname="p.NativeTest" time="0.1"/>';
test('real enumeration must be nonempty with consistent counters and expected classes', () => {
  const rows = parseJvmXml(suite(caseXml));
  requireLane(rows, ['NativeTest'], ['p.NativeTest:method']);
  assert.throws(() =>
    requireLane(rows, ['NativeTest'], ['p.NativeTest:method', 'p.NativeTest:missing']),
  );
  assert.throws(() =>
    parseJvmXml(suite(caseXml + caseXml, 'tests="2" skipped="0" failures="0" errors="0"')),
  );
  assert.throws(() => parseJvmXml(suite('', 'tests="0" skipped="0" failures="0" errors="0"')));
  assert.throws(() => requireLane(rows, ['MissingTest'], ['p.NativeTest:method']));
  assert.throws(() => parseJvmXml(suite(caseXml, 'tests="2" skipped="0" failures="0" errors="0"')));
  assert.throws(() => parseJvmXml('<!DOCTYPE x>' + suite(caseXml)));
});
test('failed and skipped outcomes are retained but cannot certify a lane', () => {
  for (const [tag, counts, expected] of [
    ['failure', 'tests="1" skipped="0" failures="1" errors="0"', 'failed'],
    ['skipped', 'tests="1" skipped="1" failures="0" errors="0"', 'skipped'],
  ]) {
    const rows = parseJvmXml(
      suite(`<testcase name="m" classname="p.NativeTest" time="0"><${tag}/></testcase>`, counts),
    );
    assert.equal(rows[0]?.status, expected);
    assert.throws(() => requireLane(rows, ['NativeTest'], ['p.NativeTest:m']));
  }
});
test('generated JVM descriptors and exact production/probe exports cannot be swapped', () => {
  const javap = Object.entries(descriptors)
    .map(([name, descriptor]) => `public native int ${name}();\n descriptor: ${descriptor}`)
    .join('\n');
  validateDescriptors(javap);
  assert.throws(() =>
    validateDescriptors(javap + '\npublic native int apiVersion();\n descriptor: ()I'),
  );

  assert.throws(() => validateDescriptors(javap.replace('()I', '()J')));
  const production = Object.keys(descriptors)
    .map((name) => `00000001 T _Java_dev_nativesketch_portable_PortableJniV1_${name}`)
    .join('\n');
  const probe =
    production +
    '\n00000002 T _Java_dev_nativesketch_portable_PendingExceptionProbe_callWithPendingException';
  validateExports(production, false);
  assert.throws(() => validateExports(production.replace('00000001 T', '         U'), false));
  validateExports(probe, true);
  assert.throws(() => validateExports(production, true));
  assert.throws(() => validateExports(probe, false));
});
