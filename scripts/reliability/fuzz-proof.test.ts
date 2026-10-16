import test from 'node:test';
import assert from 'node:assert/strict';
import { requireFuzzFlags, requireFuzzOutput } from './fuzz-proof.ts';

await test('fuzz evidence requires completed instrumented execution and rejects crash artifacts', () => {
  const text =
    'INFO: Loaded 1 modules\n#20 DONE cov: 44\nDone 20 runs in 1 second(s)\nstat::number_of_executed_units: 20';
  assert.equal(requireFuzzOutput(text).executedUnits, 20);
  for (const invalid of [
    '',
    text.replace('Done', 'Stopped'),
    text.replace('cov:', 'count:'),
    text.replace('units: 20', 'units: 1'),
    `${text}\nTest unit written to crash`,
  ])
    assert.throws(() => requireFuzzOutput(invalid));
  assert.doesNotThrow(() =>
    requireFuzzFlags({ rustflags: ['--cfg', 'fuzzing', '-Zsanitizer=address'] }),
  );
  for (const invalid of [{}, { rustflags: [] }, { rustflags: ['fuzzing'] }])
    assert.throws(() => requireFuzzFlags(invalid));
});
