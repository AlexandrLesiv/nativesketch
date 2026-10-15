import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { execFileSync, spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { checkDependencies } from './dependencies.ts';

await test('Cargo metadata exposes renamed platform dependency to boundary gate', () => {
  const directory = mkdtempSync(join(tmpdir(), 'sketch dependency ü '));
  try {
    for (const name of ['core', 'renderer']) {
      mkdirSync(join(directory, name, 'src'), { recursive: true });
      writeFileSync(join(directory, name, 'src', 'lib.rs'), '');
    }
    writeFileSync(
      join(directory, 'renderer', 'Cargo.toml'),
      '[package]\nname="skia-safe"\nversion="0.0.0"\nedition="2024"\n',
    );
    writeFileSync(
      join(directory, 'core', 'Cargo.toml'),
      '[package]\nname="sketch-core"\nversion="0.0.0"\nedition="2024"\n' +
        "[target.'cfg(windows)'.dependencies]\n" +
        'drawing={package="skia-safe",path="../renderer"}\n',
    );
    const output = execFileSync(
      'cargo',
      [
        'metadata',
        '--offline',
        '--no-deps',
        '--format-version',
        '1',
        '--manifest-path',
        join(directory, 'core', 'Cargo.toml'),
      ],
      { encoding: 'utf8', timeout: 30_000 },
    );
    const policy = {
      packages: {
        'sketch-core': {
          allowedNormalDependencies: [],
          allowedBuildDependencies: [],
        },
      },
    };
    assert.deepEqual(checkDependencies(JSON.parse(output), policy), [
      'sketch-core -> skia-safe: unapproved normal dependency',
    ]);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

await test('actual CLI rejects an untracked oversized source file', () => {
  const root = fileURLToPath(new URL('../../', import.meta.url));
  const directory = mkdtempSync(join(root, 'quality', 'negative-gate-'));
  try {
    writeFileSync(join(directory, 'oversized.rs'), '// synthetic violation\n'.repeat(201));
    const result = spawnSync(process.execPath, ['scripts/quality/check.ts'], {
      cwd: root,
      encoding: 'utf8',
      timeout: 30_000,
    });
    assert.equal(result.status, 1, result.error?.message);
    assert.match(result.stderr, /oversized\.rs: 201 lines exceeds 200/);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

await test('actual CLI rejects a legacy import in a new production Swift source', () => {
  const root = fileURLToPath(new URL('../../', import.meta.url));
  const directory = mkdtempSync(join(root, 'apps/macos/Sources/NativeSketch/negative-gate-'));
  try {
    writeFileSync(join(directory, 'LegacyImport.swift'), 'import CSketchFFI\n');
    const result = spawnSync(process.execPath, ['scripts/quality/check.ts'], {
      cwd: root,
      encoding: 'utf8',
      timeout: 30_000,
    });
    assert.equal(result.status, 1, result.error?.message);
    assert.match(result.stderr, /LegacyImport\.swift: production Swift must not import CSketchFFI/);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
