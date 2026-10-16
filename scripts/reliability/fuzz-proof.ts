import { readFile, readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { isRecord } from './validation.ts';
import { fileHash } from '../evidence/identity.ts';

export function requireFuzzOutput(text: string) {
  const completed = text.match(/Done (\d+) runs in \d+ second/);
  const executed = text.match(/stat::number_of_executed_units:\s+(\d+)/);
  if (!completed || !executed || Number(completed[1]) < 1 || completed[1] !== executed[1])
    throw new Error('Missing complete bounded fuzz execution');
  if (!/cov:\s*\d+/.test(text) || !/Loaded \d+ modules/.test(text))
    throw new Error('Missing coverage instrumentation feedback');
  if (/ERROR:|SUMMARY: AddressSanitizer|runtime error:|Test unit written to/.test(text))
    throw new Error('Fuzz failure cannot be certified');
  return { executedUnits: Number(executed[1]), completed: true };
}

export function requireFuzzFlags(value: unknown) {
  if (!isRecord(value) || !Array.isArray(value.rustflags))
    throw new Error('Missing core fuzz compiler flags');
  const flags = value.rustflags;
  if (!flags.includes('-Zsanitizer=address') || !flags.includes('fuzzing'))
    throw new Error('Core was not built with required fuzz instrumentation');
}

export async function coreFuzzProof(cache: string) {
  const directory = join(cache, 'aarch64-apple-darwin/release/.fingerprint');
  const candidates = (await readdir(directory)).filter((name) => name.startsWith('sketch-core-'));
  if (candidates.length !== 1) throw new Error('Missing or ambiguous core fuzz fingerprint');
  const path = join(directory, candidates[0], 'lib-sketch_core.json');
  const value: unknown = JSON.parse(await readFile(path, 'utf8'));
  requireFuzzFlags(value);
  return { fingerprintSha256: await fileHash(path), fingerprint: value };
}
