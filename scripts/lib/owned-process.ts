import type { ChildProcess } from 'node:child_process';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import {
  allocateOwnership,
  ownershipVariable,
  register,
  signalOwned,
  releaseOwnership,
} from './owned-registry.ts';
/** Only helper-created groups are registered; nested scopes retain cancellation isolation. */
export function spawnOwned(
  executable: string,
  args: string[],
  options: { cwd: string; env?: NodeJS.ProcessEnv; capture?: boolean },
) {
  if (process.platform === 'win32')
    throw new Error('Owned process trees require verified POSIX groups');
  const { owner, parent, root } = allocateOwnership();
  const supervisor = fileURLToPath(new URL('./owned-supervisor.ts', import.meta.url));
  const child = spawn(process.execPath, [supervisor], {
    cwd: options.cwd,
    env: { ...process.env, ...options.env, [ownershipVariable]: JSON.stringify(owner) },
    detached: true,
    shell: false,
    stdio: [
      'ignore',
      options.capture === false ? 'inherit' : 'pipe',
      options.capture === false ? 'inherit' : 'pipe',
      'ipc',
    ],
  });
  let outcome: { code: number | null; signal: ChildProcess['signalCode'] } | null = null;
  let cleanupError = '';
  let released = false;
  const kill = (signal: NodeJS.Signals) => {
    if (released) return;
    try {
      signalOwned(owner, signal);
    } catch (error) {
      cleanupError = error instanceof Error ? error.message : 'Owned tree cleanup failed';
      if (child.pid)
        try {
          process.kill(-child.pid, signal);
        } catch (failure) {
          if ((failure as NodeJS.ErrnoException).code !== 'ESRCH')
            cleanupError += '; direct supervisor cleanup failed';
        }
    }
  };
  child.once('exit', () => kill('SIGKILL'));
  child.on('message', (message: unknown) => {
    if (!message || typeof message !== 'object' || !('kind' in message)) return;
    if (message.kind === 'spawn-error')
      child.emit(
        'error',
        new Error(
          `Owned payload failed to spawn: ${'code' in message && typeof message.code === 'string' ? message.code : 'UNKNOWN'}`,
        ),
      );
    if (message.kind === 'outcome' && 'code' in message && 'signal' in message) {
      if (
        (message.code !== null && !Number.isInteger(message.code)) ||
        (message.signal !== null && typeof message.signal !== 'string')
      )
        return;
      outcome = {
        code: message.code as number | null,
        signal: message.signal as ChildProcess['signalCode'],
      };
      kill('SIGKILL');
    }
  });
  try {
    if (!child.pid) throw new Error('Owned supervisor failed to spawn');
    register(owner, parent, child.pid);
    child.send({ executable, args });
  } catch (error) {
    child.kill('SIGKILL');
    queueMicrotask(() => child.emit('error', error));
  }
  return {
    child,
    kill,
    finish(code: number | null, signal: ChildProcess['signalCode']) {
      kill('SIGKILL');
      try {
        releaseOwnership(owner, root);
      } catch (error) {
        cleanupError ||= error instanceof Error ? error.message : 'Owned registry release failed';
      }
      released = true;
      return {
        code: outcome?.code ?? code,
        signal: outcome ? outcome.signal : signal,
        error: cleanupError,
      };
    },
  };
}
