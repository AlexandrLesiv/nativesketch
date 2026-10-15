import { spawn } from 'node:child_process';

/** @typedef {{cwd:string, env?:NodeJS.ProcessEnv, timeout?:number, signal?:AbortSignal, capture?:boolean}} ProcessOptions */
/** Run argv without a shell. Captured diagnostics are bounded; children are reaped. */
/** @param {string} executable @param {string[]} args @param {ProcessOptions} options */
export function run(executable, args, options) {
  if (/\.(cmd|bat)$/i.test(executable)) {
    throw new Error('Vendor batch launchers require a native executable or Node CLI entry point');
  }
  return new Promise((resolve, reject) => {
    if (options.signal?.aborted) {
      reject(new Error('Process cancelled'));
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
    /** @type {NodeJS.Timeout | undefined} */
    let killTimer;
    const stop = () => {
      if (stopping) return;
      stopping = true;
      reason ||= 'Process cancelled';
      const pid = child.pid;
      if (!pid) return;
      const terminate = (/** @type {NodeJS.Signals} */ signal) => {
        try {
          if (process.platform !== 'win32') process.kill(-pid, signal);
          else child.kill(signal);
        } catch (error) {
          if (/** @type {NodeJS.ErrnoException} */ (error).code !== 'ESRCH')
            reason += ': cleanup failed';
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
    const append = (/** @type {Buffer} */ data) => {
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
    child.once('error', reject);
    child.once('close', (code, signal) => {
      clearTimeout(timer);
      if (reason && child.pid && process.platform !== 'win32') {
        try {
          process.kill(-child.pid, 'SIGKILL');
        } catch (error) {
          if (/** @type {NodeJS.ErrnoException} */ (error).code !== 'ESRCH')
            reason += ': descendant cleanup failed';
        }
      }
      clearTimeout(killTimer);
      options.signal?.removeEventListener('abort', stop);
      if (reason || code !== 0)
        reject(
          new Error(`${executable}: ${reason || `exit ${code}, signal ${signal}`}\n${output}`),
        );
      else resolve(output.trim());
    });
  });
}
