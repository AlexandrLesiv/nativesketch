import { spawn } from 'node:child_process';
import { ownership, ownershipVariable, assertMayStart, signalOwned } from './owned-registry.ts';
const owner = ownership(process.env[ownershipVariable] ?? '');
let started = false;
// Keep the group leader alive until its parent has retained the exact payload outcome.
process.on('SIGTERM', () => {});
process.once('disconnect', () => {
  try {
    signalOwned(owner, 'SIGKILL');
  } finally {
    process.kill(-process.pid, 'SIGKILL');
  }
});
process.once('message', (value: unknown) => {
  if (
    started ||
    !value ||
    typeof value !== 'object' ||
    !('executable' in value) ||
    !('args' in value) ||
    typeof value.executable !== 'string' ||
    !Array.isArray(value.args) ||
    !value.args.every((item) => typeof item === 'string')
  )
    throw new Error('Invalid supervisor start');
  started = true;
  assertMayStart(owner);
  const child = spawn(value.executable, value.args, {
    shell: false,
    detached: false,
    stdio: ['ignore', 'inherit', 'inherit'],
  });
  child.once('error', (error: NodeJS.ErrnoException) => {
    process.send?.({ kind: 'spawn-error', code: error.code ?? 'UNKNOWN' });
    process.send?.({ kind: 'outcome', code: null, signal: null });
  });
  child.once('exit', (code, signal) => process.send?.({ kind: 'outcome', code, signal }));
});
