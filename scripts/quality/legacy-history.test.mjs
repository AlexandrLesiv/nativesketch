import assert from 'node:assert/strict';
import { test } from 'node:test';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { checkLegacyGrowth, legacyChanges } from './legacy-growth.mjs';

await test('real history catches regrowth hidden behind a later documentation commit', () => {
  const directory = mkdtempSync(join(tmpdir(), 'sketch history ü '));
  /** @param {...string} args */
  const git = (...args) =>
    execFileSync(
      'git',
      [
        '-c',
        'user.name=Fixture',
        '-c',
        'user.email=fixture@example.invalid',
        '-c',
        'commit.gpgsign=false',
        '-c',
        `core.hooksPath=${join(directory, 'no-hooks')}`,
        ...args,
      ],
      { cwd: directory, encoding: 'utf8', timeout: 30_000 },
    );
  const source = (/** @type {number} */ lines) => '// fixture\n'.repeat(lines);
  const commit = (/** @type {string} */ message) => {
    git('add', '.');
    git('commit', '--quiet', '-m', message);
  };
  try {
    git('init', '--quiet', '--template=');
    writeFileSync(join(directory, 'old.rs'), source(210));
    commit('baseline');
    const baseline = git('rev-parse', 'HEAD').trim();
    writeFileSync(join(directory, 'old.rs'), source(201));
    commit('shrink');
    writeFileSync(join(directory, 'old.rs'), source(205));
    commit('regrow');
    writeFileSync(join(directory, 'README.md'), 'later documentation\n');
    commit('documentation');
    const changes = legacyChanges(
      git,
      baseline,
      ['old.rs'],
      [{ path: 'old.rs', text: source(205) }],
    );
    const failures = checkLegacyGrowth(changes, 200);
    assert.equal(failures.length, 1);
    assert.match(failures[0], /201 -> 205/);
    writeFileSync(join(directory, 'old.rs'), source(201));
    commit('repair');
    assert.deepEqual(
      checkLegacyGrowth(
        legacyChanges(git, baseline, ['old.rs'], [{ path: 'old.rs', text: source(201) }]),
        200,
      ),
      [],
    );
    git('checkout', '--quiet', '--orphan', 'synthetic-main');
    git('rm', '--quiet', '-rf', '.');
    writeFileSync(join(directory, 'README.md'), 'unrelated main\n');
    commit('unrelated main');
    git('merge', '--quiet', '--allow-unrelated-histories', '--no-edit', baseline);
    assert.deepEqual(
      checkLegacyGrowth(
        legacyChanges(git, baseline, ['old.rs'], [{ path: 'old.rs', text: source(210) }]),
        200,
      ),
      [],
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
