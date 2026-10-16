import type { Discovery, Execution, Result } from '../evidence/types.ts';
import { randomUUID } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { command, requireHost, root } from '../lib/context.ts';
import { fileHash, sourceIdentity } from '../evidence/identity.ts';

const caseName = 'legacy-abi2-conformance';
const testId = `c-abi:sketch-ffi:macos:${caseName}`;

/** Actual C header/compiler/link/runtime acceptance; never opens an application. */
export async function checkLegacyFfi(destination?: string, execute = command) {
  requireHost('darwin');
  const output = resolve(root, destination ?? `artifacts/evidence/legacy-ffi-${randomUUID()}`);
  await mkdir(output, { recursive: true });
  // Refuse to overwrite any completed or interrupted attempt.
  await writeFile(join(output, 'attempt.lock'), new Date().toISOString(), { flag: 'wx' });
  const source = await sourceIdentity(root);
  const logs: string[] = [];
  const json = (name: string, value: unknown) =>
    writeFile(join(output, name), `${JSON.stringify(value, null, 2)}\n`);
  const run = async (name: string, executable: string, args: string[]) => {
    const path = `${name}.log`;
    logs.push(path);
    try {
      const text = await execute(executable, args, { capture: true });
      await writeFile(join(output, path), `${text}\n`);
      return text;
    } catch (error) {
      await writeFile(join(output, path), String(error));
      throw error;
    }
  };
  const discovery: Discovery = {
    schemaVersion: 1,
    source,
    tests: [],
  };
  const execution: Execution = { schemaVersion: 1, source, results: [] };
  try {
    const toolchain = await run('toolchain', 'clang', ['--version']);
    const rust = await run('rust', 'rustc', ['--version', '--verbose']);
    const os = await run('os', 'sw_vers', []);
    const model = await run('device', 'sysctl', ['-n', 'hw.model']);
    await run('build', 'cargo', ['build', '--locked', '-p', 'sketch-ffi', '--release']);
    const library = join(root, 'target/release/libsketch_ffi.a');
    const header = join(root, 'crates/sketch-ffi/include/sketch_ffi.h');
    const fixture = join(root, 'crates/sketch-core/tests/fixtures/portable-v1.nsketch');
    const inputs = [library, header, fixture];
    const inputHashes = await Promise.all(inputs.map(fileHash));
    const binary = join(output, 'consumer');
    await run('compile', 'clang', [
      '-std=c11',
      '-DNDEBUG',
      '-Wall',
      '-Wextra',
      '-Werror',
      '-I',
      join(root, 'crates/sketch-ffi/include'),
      join(root, 'crates/sketch-ffi/tests/consumer.c'),
      library,
      '-framework',
      'Security',
      '-o',
      binary,
    ]);
    const binarySha256 = inputHashes[0];
    const testBinarySha256 = await fileHash(binary);
    await json('identity.json', {
      source,
      binarySha256,
      testBinarySha256,
      inputs: inputs.map((path, index) => ({ path, sha256: inputHashes[index] })),
      toolchain,
      rust,
      node: process.version,
      os,
      model,
      arch: process.arch,
    });
    const listed = await run('discovery', binary, ['--list']);
    if (listed !== caseName) throw new Error('Unexpected actual C consumer enumeration');
    discovery.scopes = [{ runner: 'c-abi', platform: 'macos', level: 'integration' }];
    discovery.tests.push({
      id: testId,
      runner: 'c-abi',
      platform: 'macos',
      level: 'integration',
      pointerWidth: 64,
      binarySha256,
      testBinarySha256,
    });
    const started = Date.now();
    const result: Result = {
      testId,
      attempt: 1,
      status: 'failed',
      durationMs: 0,
      timestamp: new Date().toISOString(),
      binarySha256,
      testBinarySha256,
      platform: 'macos',
      toolchain: `${toolchain}\n${rust}\nNode ${process.version}`,
      device: `${model}/${process.arch}`,
      os,
      environment: { valid: true, reasons: [] },
      artifacts: logs,
      inputRoute: 'C compiler/header and statically linked legacy ABI2',
    };
    execution.results.push(result);
    try {
      const expectFailure = async (name: string, args: string[], marker: string) => {
        try {
          await run(name, binary, args);
        } catch (error) {
          if (!String(error).includes('exit 1,') || !String(error).includes(marker)) throw error;
          return;
        }
        throw new Error(`${name}: deliberate mismatch unexpectedly passed`);
      };
      await expectFailure('negative-call', ['--negative-oracle'], 'sketch_cancel(NULL) == 0');
      const corrupt = join(output, 'corrupt-fixture.nsketch');
      const bytes = await readFile(fixture);
      bytes[0] ^= 0xff;
      await writeFile(corrupt, bytes);
      await expectFailure(
        'negative-bytes',
        ['--run', caseName, corrupt],
        'memcmp(actual, expected',
      );
      const text = await run('execution', binary, ['--run', caseName, fixture]);
      if (text !== `PASS ${caseName}`) throw new Error('Consumer did not report completion');
      const unchanged =
        JSON.stringify(await sourceIdentity(root)) === JSON.stringify(source) &&
        JSON.stringify(await Promise.all(inputs.map(fileHash))) === JSON.stringify(inputHashes) &&
        (await fileHash(binary)) === testBinarySha256;
      if (!unchanged) {
        result.environment = { valid: false, reasons: ['Source or binary changed during run'] };
        throw new Error('Source or binary changed during legacy ABI2 check');
      }
      result.status = 'passed';
      result.output = text;
      console.log(`PASS: real legacy ABI2 C consumer; evidence ${output}`);
    } finally {
      result.durationMs = Date.now() - started;
      result.artifacts = [...logs, 'identity.json', 'consumer'];
    }
  } catch (error) {
    await json('failure.json', { source, error: String(error) });
    throw error;
  } finally {
    await json('discovery.json', discovery);
    await json('results.json', execution);
  }
  return output;
}
