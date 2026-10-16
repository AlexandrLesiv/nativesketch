import { spawn } from 'node:child_process';
import { spawnOwned } from './owned-process.ts';
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
    const owned =
      process.platform === 'win32'
        ? null
        : spawnOwned(executable, args, { ...options, capture: options.capture ?? false });
    const child =
      owned?.child ??
      spawn(executable, args, {
        cwd: options.cwd,
        env: { ...process.env, ...options.env },
        shell: false,
        stdio: options.capture ? ['ignore', 'pipe', 'pipe'] : 'inherit',
      });
    let output = '';
    let reason = '';
    let stopping = false;
    let capturedBytes = 0;

    let killTimer: ReturnType<typeof setTimeout> | undefined;
    const stop = () => {
      if (stopping) return;
      stopping = true;
      reason ||= 'Process cancelled';
      const pid = child.pid;
      if (!pid) return;
      const terminate = (signal: NodeJS.Signals) => {
        try {
          if (owned) owned.kill(signal);
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
    child.once('close', (supervisorCode, supervisorSignal) => {
      clearTimeout(timer);
      const outcome = owned?.finish(supervisorCode, supervisorSignal);
      const code = outcome ? outcome.code : supervisorCode;
      const signal = outcome ? outcome.signal : supervisorSignal;
      reason ||= outcome?.error ?? '';
      clearTimeout(killTimer);
      options.signal?.removeEventListener('abort', stop);
      if (reason || code !== 0)
        reject(new ProcessFailure(executable, code, signal, reason, output));
      else resolve(output.trim());
    });
  });
}
