import test from 'node:test';
import assert from 'node:assert/strict';
import { e2eOptions } from '../e2e/options.ts';
import { selectedTests, selectionArguments, nativeInputRoute } from '../e2e/selection.ts';

const first = 'xctest:NativeSketchUITests:NativeGestureCancellationTests/testResizeCancelsDraft';
const second =
  'xctest:NativeSketchUITests:NativeGestureCancellationTests/testDeactivationCancelsDraft';
await test('scoped native execution preserves full discovery and never enables foreground implicitly', () => {
  const discovery = [first, second];
  const options = e2eOptions(['--platform', 'macos', '--only', second]);
  assert.equal(options.execute, false);
  assert.deepEqual(selectedTests(discovery, options.only), [second]);
  assert.deepEqual(discovery, [first, second]);
  assert.deepEqual(selectedTests(discovery, []), discovery);
  assert.deepEqual(selectionArguments([second]), [
    '-only-testing:NativeSketchUITests/NativeGestureCancellationTests/testDeactivationCancelsDraft',
  ]);
});
await test('unknown, duplicate and malformed native selections fail before execution', () => {
  assert.throws(() => selectedTests([first], [second]), /not discovered/);
  assert.throws(() => selectedTests([first], [first, first]), /Duplicate/);
  assert.throws(() => selectedTests([first], ['NativeSketchUITests']), /Invalid/);
  assert.throws(() => selectedTests([first], ['xctest:Other:Tests/testCase']), /Invalid/);
});
await test('native evidence distinguishes hybrid gesture input from XCTest-only journeys', () => {
  assert.match(nativeInputRoute(first), /CGEvent/);
  assert.doesNotMatch(
    nativeInputRoute('xctest:NativeSketchUITests:EditorJourneyTests/testDrawSaveReopen'),
    /CGEvent/,
  );
});
