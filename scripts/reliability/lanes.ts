import { sanitizerEnvironment, sanitizerProof } from './asan-proof.ts';
import { readdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { homedir } from 'node:os';
import { controller } from '../lib/context.ts';
import { fileHash } from '../evidence/identity.ts';
import { recorded, json } from './evidence.ts';
import {
  requireVersion,
  requireUnchanged,
  validateAdvisories,
  validateTestOutput,
} from './validation.ts';

export interface Pins {
  node: string;
  cargoDeny: string;
  nightly: string;
  rustcCommit: string;
  maximumDatabaseAgeDays: number;
  ffiTests: number;
}
export async function executeLane(
  lane: string,
  root: string,
  output: string,
  tools: string,
  pins: Pins,
) {
  const env = {
    RUSTUP_HOME: join(tools, 'rustup'),
    CARGO_TERM_COLOR: 'never',
    RUSTFLAGS: '',
    CARGO_ENCODED_RUSTFLAGS: '',
    RUSTDOCFLAGS: '',
    MIRIFLAGS: '',
    ASAN_OPTIONS: '',
  };
  const command = (name: string, executable: string, args: string[], extra = {}) =>
    recorded(output, name, executable, args, {
      cwd: root,
      env: { ...env, ...extra },
      timeout: 1_200_000,
      signal: controller.signal,
    });
  if (lane === 'advisories') {
    const executable = join(tools, 'cargo-tools/bin/cargo-deny');
    const version = await command('tool-version', executable, ['--version']);
    requireVersion(version, pins.cargoDeny);
    const directory = join(tools, 'advisory-dbs');
    const databases = (await readdir(directory)).filter((name) => name.startsWith('advisory-db-'));
    if (databases.length !== 1) throw new Error('Expected one provisioned RustSec database');
    const database = join(directory, databases[0]);
    const git = async (name: string, args: string[]) =>
      recorded(output, name, 'git', args, {
        cwd: database,
        timeout: 30_000,
        signal: controller.signal,
      });
    const revision = await git('database-revision', ['rev-parse', 'HEAD']);
    const date = await git('database-date', ['show', '-s', '--format=%cI', 'HEAD']);
    if (await git('database-status', ['status', '--porcelain']))
      throw new Error('Advisory database is dirty');
    const ageDays = (Date.now() - Date.parse(date)) / 86_400_000;
    if (!Number.isFinite(ageDays) || ageDays < 0) throw new Error('Invalid advisory database date');
    await json(output, 'tools.json', {
      executable,
      sha256: await fileHash(executable),
      version,
      database: { revision, commitDate: date, commitAgeDays: ageDays },
      freshnessPolicy: `cargo-deny maximum-db-staleness P${pins.maximumDatabaseAgeDays}D; offline, no refresh`,
    });
    const config = join(output, 'deny.toml');
    await writeFile(
      config,
      `[advisories]\ndb-path = ${JSON.stringify(directory)}\nyanked = "deny"\nmaximum-db-staleness = "P${pins.maximumDatabaseAgeDays}D"\n`,
    );
    const text = await command('execution', executable, [
      '--offline',
      '--locked',
      '--all-features',
      '--format',
      'json',
      '--config',
      config,
      'check',
      '--show-stats',
      'advisories',
    ]);
    requireUnchanged(revision, await git('database-final-revision', ['rev-parse', 'HEAD']));
    return {
      advisories: validateAdvisories(text),
      scope: 'Locked workspace dependency advisories only; not license compliance',
    };
  }
  if (process.platform !== 'darwin' || process.arch !== 'arm64')
    throw new Error('Miri/ASan lane currently verified only on macOS arm64');
  const rustup = join(homedir(), '.cargo/bin/rustup');
  const bin = join(tools, `rustup/toolchains/${pins.nightly}-aarch64-apple-darwin/bin`);
  const rustc = await command('rustc-version', rustup, [
    'run',
    pins.nightly,
    'rustc',
    '--version',
    '--verbose',
  ]);
  if (!rustc.includes(`commit-hash: ${pins.rustcCommit}`))
    throw new Error('Pinned nightly compiler mismatch');
  await json(output, 'tools.json', {
    nightly: pins.nightly,
    rustc,
    cargoSha256: await fileHash(join(bin, 'cargo')),
    rustcSha256: await fileHash(join(bin, 'rustc')),
    ...(lane === 'miri' ? { miriSha256: await fileHash(join(bin, 'miri')) } : {}),
  });
  const args =
    lane === 'miri'
      ? ['miri', 'test', '--locked', '-p', 'sketch-ffi', '--lib']
      : [
          'test',
          '--locked',
          '-Zbuild-std',
          '--message-format=json',
          '--target',
          'aarch64-apple-darwin',
          '-p',
          'sketch-ffi',
          '--lib',
        ];
  const text = await command('execution', rustup, ['run', pins.nightly, 'cargo', ...args], {
    CARGO_TARGET_DIR: join(root, 'target', `reliability-${lane}`),
    ...(lane === 'asan' ? sanitizerEnvironment() : {}),
  });
  return {
    testIds: validateTestOutput(text, pins.ffiTests),
    ...(lane === 'asan'
      ? { instrumentation: await sanitizerProof(root, output, tools, pins.nightly, rustup, text) }
      : {}),
    scope: 'Legacy Rust FFI/core only; no Skia, native UI, JNI, Wasm or portable facade',
  };
}
