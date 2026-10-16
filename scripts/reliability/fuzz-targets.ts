import { join } from 'node:path';
import { fileHash } from '../evidence/identity.ts';
import { coreFuzzProof } from './fuzz-proof.ts';
import { requireInstrumentation } from './asan-proof.ts';

export type FuzzCommand = (
  name: string,
  tool: string,
  args: string[],
  timeout?: number,
) => Promise<string>;
export const targetNames = ['document_codec', 'command_sequence'];

export async function buildFuzzTargets(root: string, executable: string, command: FuzzCommand) {
  const directory = join(root, 'fuzz');
  const cache = join(root, 'target/reliability-fuzz');
  const listed = (
    await command('target-list', executable, ['fuzz', 'list', '--fuzz-dir', directory])
  )
    .split(/\s+/)
    .filter(Boolean)
    .sort();
  if (JSON.stringify(listed) !== JSON.stringify([...targetNames].sort()))
    throw new Error('Fuzz target enumeration differs from the supported lane');
  const targets = [];
  for (const name of targetNames) {
    await command(`${name}-build`, executable, [
      'fuzz',
      'build',
      name,
      '--fuzz-dir',
      directory,
      '--target-dir',
      cache,
      '--sanitizer',
      'address',
      '--verbose',
    ]);
    const binary = join(cache, 'aarch64-apple-darwin/release', name);
    const symbols = await command(`${name}-symbols`, '/usr/bin/nm', ['-g', binary]);
    requireInstrumentation(symbols);
    targets.push({
      name,
      binary,
      binarySha256: await fileHash(binary),
      core: await coreFuzzProof(cache),
    });
  }
  return targets;
}
