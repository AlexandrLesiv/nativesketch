import { readFile, mkdir } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { homedir, release } from 'node:os';
import { root, main, controller } from './lib/context.ts';
import { fileHash, sourceIdentity } from './evidence/identity.ts';
import { recorded, freshOutput, json } from './reliability/evidence.ts';
import { requireVersion, requireUnchanged } from './reliability/validation.ts';
import { buildFuzzTargets } from './reliability/fuzz-targets.ts';
import { executeFuzzTargets } from './reliability/fuzz-execution.ts';
import { fuzzDiscovery } from './reliability/fuzz-report.ts';

await main(async () => {
  const args = process.argv.slice(2);
  const attempt = args.length === 4 && args[2] === '--attempt' ? Number(args[3]) : 1;
  if (
    ![2, 4].includes(args.length) ||
    args[0] !== '--output' ||
    !args[1] ||
    (args.length === 4 && args[2] !== '--attempt') ||
    !Number.isSafeInteger(attempt) ||
    attempt < 1
  )
    throw new Error('Usage: node scripts/check-fuzz.ts --output NEW_DIRECTORY [--attempt N]');
  if (process.platform !== 'darwin' || process.arch !== 'arm64')
    throw new Error('This fuzz lane is currently verified only on macOS arm64');
  const output = resolve(args[1]);
  await freshOutput(output);
  const source = await sourceIdentity(root);
  const startedAt = new Date().toISOString();
  const pins: { node: string; nightly: string; rustcCommit: string; cargoFuzz: string } =
    JSON.parse(await readFile(join(root, 'quality/reliability-tools.json'), 'utf8'));
  const tools = join(root, '.build-cache/toolchains');
  const executable = join(tools, 'cargo-fuzz/bin/cargo-fuzz');
  const rustup = join(homedir(), '.cargo/bin/rustup');
  const env = {
    RUSTUP_HOME: join(tools, 'rustup'),
    RUSTUP_TOOLCHAIN: pins.nightly,
    CARGO_NET_OFFLINE: 'true',
    CARGO_ENCODED_RUSTFLAGS: undefined,
    RUSTFLAGS: undefined,
    ASAN_OPTIONS: 'detect_leaks=1:exitcode=86:abort_on_error=0',
  };
  const command = (name: string, tool: string, argv: string[], timeout = 1_200_000) =>
    recorded(output, name, tool, argv, {
      cwd: root,
      env,
      timeout,
      signal: controller.signal,
    });
  await json(output, 'identity.json', {
    source,
    startedAt,
    platform: process.platform,
    os: release(),
  });
  try {
    if (process.versions.node !== pins.node) throw new Error('Pinned Node version required');
    const version = await command('tool-version', executable, ['fuzz', '--version']);
    requireVersion(version, pins.cargoFuzz);
    const rustc = await command('rustc-version', rustup, ['run', pins.nightly, 'rustc', '-Vv']);
    if (!rustc.includes(`commit-hash: ${pins.rustcCommit}`))
      throw new Error('Pinned nightly compiler mismatch');
    await json(output, 'tools.json', {
      version,
      rustc,
      cargoFuzzSha256: await fileHash(executable),
      clearedEnvironment: ['CARGO_ENCODED_RUSTFLAGS', 'RUSTFLAGS'],
    });
    await mkdir(join(output, 'crashes'));
    const targets = await buildFuzzTargets(root, executable, command);
    requireUnchanged(source, await sourceIdentity(root));
    await fuzzDiscovery(output, source, targets);
    const results = await executeFuzzTargets(
      root,
      output,
      source,
      rustc,
      attempt,
      targets,
      command,
    );
    requireUnchanged(source, await sourceIdentity(root));
    controller.signal.throwIfAborted();
    await json(output, 'result.json', {
      status: 'passed',
      source,
      startedAt,
      finishedAt: new Date().toISOString(),
      results,
      scope:
        'Bounded codec and typed command/input fuzz smoke, not exhaustive or native UI coverage',
    });
    controller.signal.throwIfAborted();
  } catch (error) {
    await json(output, 'result.json', {
      status: 'failed',
      source,
      startedAt,
      finishedAt: new Date().toISOString(),
      reason: String(error),
    });
    throw error;
  }
});
