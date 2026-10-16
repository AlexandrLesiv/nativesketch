import { spawn } from 'node:child_process';
import { ProcessFailure } from './process-failure.ts';

export type ProcessOptions = {
  cwd: string;
  env?: NodeJS.ProcessEnv;
  timeout?: number;
  signal?: AbortSignal;
  capture?: boolean;
};
/** Run argv without a shell. Captured diagnostics are bounded; children are reaped. */

export function run(executable: string, args: string[], options: ProcessOptions) {
  if (/\.(cmd|bat)$/i.test(executable)) {
    throw new Error('Vendor batch launchers require a native executable or Node CLI entry point');
  }
  return new Promise<string>((resolve, reject) => {
    if (options.signal?.aborted) {
      reject(new ProcessFailure(executable, null, null, 'Process cancelled', ''));
      return;
    }
    const child = spawn(executable, args, {
      cwd: options.cwd,
      env: { ...process.env, ...options.env },
      shell: false,
      detached: process.platform !== 'win32',
      stdio: options.capture ? ['ignore', 'pipe', 'pipe'] : 'inherit',
    });
    let output = '';
    let reason = '';
    let stopping = false;
    let capturedBytes = 0;

    let killTimer: NodeJS.Timeout | undefined;
    const stop = () => {
      if (stopping) return;
      stopping = true;
      reason ||= 'Process cancelled';
      const pid = child.pid;
      if (!pid) return;
      const terminate = (signal: NodeJS.Signals) => {
        try {
          if (process.platform !== 'win32') process.kill(-pid, signal);
          else child.kill(signal);
        } catch (error) {
          if ((error as NodeJS.ErrnoException).code !== 'ESRCH') reason += ': cleanup failed';
        }
      };
      terminate('SIGTERM');
      killTimer = setTimeout(() => terminate('SIGKILL'), 1000);
    };
    const timer = setTimeout(() => {
      reason = 'Process timed out';
      stop();
    }, options.timeout ?? 600_000);
    options.signal?.addEventListener('abort', stop, { once: true });
    const append = (data: Buffer) => {
      const remaining = Math.max(0, 4_000_000 - capturedBytes);
      output += data.subarray(0, remaining).toString();
      capturedBytes += Math.min(data.length, remaining);
      if (data.length > remaining) {
        reason = 'Process output exceeded limit';
        stop();
      }
    };
    child.stdout?.on('data', append);
    child.stderr?.on('data', append);
    child.once('error', (error) => {
      reason = `Process spawn failed: ${error.message}`;
    });
    child.once('close', (code, signal) => {
      clearTimeout(timer);
      if (reason && child.pid && process.platform !== 'win32') {
        try {
          process.kill(-child.pid, 'SIGKILL');
        } catch (error) {
          if ((error as NodeJS.ErrnoException).code !== 'ESRCH')
            reason += ': descendant cleanup failed';
        }
      }
      clearTimeout(killTimer);
      options.signal?.removeEventListener('abort', stop);
      if (reason || code !== 0)
        reject(new ProcessFailure(executable, code, signal, reason, output));
      else resolve(output.trim());
    });
  });
}
