import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { spawnOwned } from '../lib/owned-process.ts';
import { ProcessFailure } from '../lib/process-failure.ts';
/** Keep reporter stdout isolated; cancellation owns the complete POSIX worker group. */
export async function nodeProcess(root: string, args: string[], signal?: AbortSignal) {
  signal?.throwIfAborted();
  const env = { ...process.env, NODE_TEST_CONTEXT: undefined };
  if (process.platform === 'win32') {
    if (signal)
      throw new Error('Cancellable Node evidence requires verified Windows process-tree ownership');
    try {
      const result = await promisify(execFile)(process.execPath, args, {
        cwd: root,
        env,
        timeout: 600000,
        maxBuffer: 4000000,
      });
      return { output: result.stdout, exitFailed: false };
    } catch (error) {
      const failure = error as Error & { code?: number; stdout?: string };
      if (failure.code !== 1 || !failure.stdout) throw error;
      return { output: failure.stdout, exitFailed: true };
    }
  }
  return new Promise<{ output: string; exitFailed: boolean }>((resolve, reject) => {
    const owned = spawnOwned(process.execPath, args, { cwd: root, env });
    const child = owned.child;
    const stdout: Buffer[] = [];
    const stderr: Buffer[] = [];
    let outBytes = 0;
    let errBytes = 0;
    let reason = '';
    let stopping = false;
    let killTimer: ReturnType<typeof setTimeout> | undefined;
    const kill = owned.kill;
    const stop = () => {
      if (stopping) return;
      stopping = true;
      kill('SIGTERM');
      killTimer = setTimeout(() => kill('SIGKILL'), 500);
    };
    const cancel = () => {
      reason = 'Node evidence cancelled';
      stop();
    };
    const timer = setTimeout(() => {
      reason = 'Node evidence timed out';
      stop();
    }, 600000);
    signal?.addEventListener('abort', cancel, { once: true });
    const append = (chunk: Buffer, stream: 'out' | 'err') => {
      const used = stream === 'out' ? outBytes : errBytes;
      const captured = chunk.subarray(0, Math.max(0, 4000000 - used));
      if (captured.length) (stream === 'out' ? stdout : stderr).push(captured);
      if (stream === 'out') outBytes += captured.length;
      else errBytes += captured.length;
      if (chunk.length > captured.length) {
        reason = 'Node evidence output limit exceeded';
        stop();
      }
    };
    child.stdout?.on('data', (chunk: Buffer) => append(chunk, 'out'));
    child.stderr?.on('data', (chunk: Buffer) => append(chunk, 'err'));
    child.on('error', () => {
      reason = 'Node evidence worker failed to spawn';
    });
    child.on('exit', stop);
    child.on('close', (supervisorCode, supervisorSignal) => {
      const outcome = owned.finish(supervisorCode, supervisorSignal);
      const code = outcome.code;
      const exitSignal = outcome.signal;
      reason ||= outcome.error;
      clearTimeout(timer);
      clearTimeout(killTimer);
      kill('SIGKILL');
      signal?.removeEventListener('abort', cancel);
      const output = Buffer.concat(stdout).toString();
      if (signal?.aborted) reason = 'Node evidence cancelled';
      if (reason || exitSignal || (code !== 0 && code !== 1))
        reject(
          new ProcessFailure(
            process.execPath,
            code,
            exitSignal,
            reason,
            output + Buffer.concat(stderr).toString(),
          ),
        );
      else resolve({ output, exitFailed: code === 1 });
    });
    if (signal?.aborted) cancel();
  });
}
