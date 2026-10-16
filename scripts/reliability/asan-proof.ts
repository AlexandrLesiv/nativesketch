import { realpath, writeFile } from 'node:fs/promises';
import { join, relative, isAbsolute } from 'node:path';
import { fileHash } from '../evidence/identity.ts';
import { ProcessFailure } from '../lib/process-failure.ts';
import { recorded, json } from './evidence.ts';
import { isRecord } from './validation.ts';

/** Cargo gives encoded flags precedence even when the encoded value is empty. */
export function sanitizerEnvironment() {
  return {
    RUSTFLAGS: '',
    CARGO_ENCODED_RUSTFLAGS: '-Zsanitizer=address',
    ASAN_OPTIONS: 'detect_leaks=1:exitcode=86:abort_on_error=0',
  };
}
export function instrumentedArtifact(log: string): string {
  const artifacts: string[] = [];
  for (const line of log.split('\n')) {
    if (!line.startsWith('{')) continue;
    const value: unknown = JSON.parse(line);
    if (
      !isRecord(value) ||
      value.reason !== 'compiler-artifact' ||
      !isRecord(value.target) ||
      value.target.name !== 'sketch_ffi' ||
      !isRecord(value.profile) ||
      value.profile.test !== true
    )
      continue;
    if (typeof value.executable === 'string') artifacts.push(value.executable);
  }
  if (artifacts.length !== 1) throw new Error('Missing or ambiguous sanitizer test binary');
  return artifacts[0];
}
export function requireInstrumentation(symbols: string) {
  if (!/\b___asan_init\b/.test(symbols) || !/\b___asan_report_load\d+\b/.test(symbols))
    throw new Error('Test binary lacks AddressSanitizer instrumentation');
}
export function requireCanaryFailure(error: unknown) {
  if (
    !(error instanceof ProcessFailure) ||
    error.exitCode !== 86 ||
    error.signal ||
    error.reason ||
    !error.output.includes('ERROR: AddressSanitizer: heap-use-after-free') ||
    !error.output.includes('READ of size 1')
  )
    throw new Error('Canary did not demonstrate the expected sanitizer detection');
}
export async function sanitizerProof(
  root: string,
  output: string,
  tools: string,
  nightly: string,
  rustup: string,
  log: string,
) {
  const cache = await realpath(join(root, 'target/reliability-asan'));
  const binary = await realpath(instrumentedArtifact(log));
  const path = relative(cache, binary);
  if (path.startsWith('..') || isAbsolute(path))
    throw new Error('Test binary outside sanitizer cache');
  const env = { RUSTUP_HOME: join(tools, 'rustup'), ...sanitizerEnvironment() };
  const symbols = await recorded(output, 'asan-symbols', '/usr/bin/nm', ['-g', binary], {
    cwd: root,
  });
  requireInstrumentation(symbols);
  const source = join(output, 'asan-canary.rs');
  await writeFile(
    source,
    'fn main() {\n    let mut bytes = vec![7u8; 16];\n    let pointer = bytes.as_mut_ptr();\n    drop(bytes);\n    unsafe { std::ptr::read_volatile(pointer); }\n}\n',
  );
  const executable = join(output, 'asan-canary');
  await recorded(
    output,
    'canary-build',
    rustup,
    [
      'run',
      nightly,
      'rustc',
      source,
      '--crate-name',
      'asan_canary',
      '--target',
      'aarch64-apple-darwin',
      '-Zsanitizer=address',
      '-C',
      'opt-level=0',
      '-g',
      '-o',
      executable,
    ],
    { cwd: root, env },
  );
  let failure: unknown;
  try {
    await recorded(output, 'canary-execution', executable, [], { cwd: root, env, timeout: 30_000 });
  } catch (error) {
    failure = error;
  }
  requireCanaryFailure(failure);
  const proof = {
    binary,
    binarySha256: await fileHash(binary),
    instrumentation: 'ASan initialization and load-report symbols',
    canary: {
      expectedFailure: 'heap-use-after-free READ of size 1',
      exitCode: 86,
      binarySha256: await fileHash(executable),
      sourceSha256: await fileHash(source),
    },
  };
  await json(output, 'instrumentation.json', proof);
  return proof;
}
