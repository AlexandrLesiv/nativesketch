import { StringDecoder } from 'node:string_decoder';
import { spawnOwned } from '../lib/owned-process.ts';
export interface OwnedResult {
  exitCode: number | null;
  signal: string | null;
  reason: string | null;
  output: string;
  bytes: number;
}
/** Only the spawned process group is owned. Complete after termination and pipe closure. */
export function runOwned(
  executable: string,
  args: string[],
  cwd: string,
  signal: AbortSignal,
  onOutput: (output: string, bytes: number) => void,
  timeout = 1_200_000,
): Promise<OwnedResult> {
  if (process.platform === 'win32') throw new Error('Windows process tree ownership unverified');
  return new Promise((resolve) => {
    const decoder = new StringDecoder('utf8');
    let output = '';
    let bytes = 0;
    let reason: string | null = null;
    if (signal.aborted) {
      resolve({ exitCode: null, signal: null, reason: 'Cancelled', output, bytes });
      return;
    }
    const owned = spawnOwned(executable, args, { cwd });
    const child = owned.child;
    let killTimer: ReturnType<typeof setTimeout> | undefined;
    let stopping = false;
    const kill = owned.kill;
    const terminate = () => {
      if (stopping) return;
      stopping = true;
      kill('SIGTERM');
      killTimer ??= setTimeout(() => kill('SIGKILL'), 500);
    };
    const cancel = () => {
      reason = 'Cancelled';
      terminate();
    };
    const timer = setTimeout(() => {
      reason = 'Timed out';
      terminate();
    }, timeout);
    signal.addEventListener('abort', cancel, { once: true });
    const append = (buffer: Buffer) => {
      const available = Math.max(0, 1024 * 1024 - bytes);
      output += decoder.write(buffer.subarray(0, available));
      bytes += Math.min(buffer.length, available);
      const expanded = Buffer.byteLength(output) > 1024 * 1024;
      if (expanded)
        output = new StringDecoder('utf8').write(Buffer.from(output).subarray(0, 1024 * 1024));
      onOutput(output, bytes);
      if (buffer.length > available || expanded) {
        reason = 'Output exceeded 1 MiB';
        terminate();
      }
    };
    child.stdout?.on('data', append);
    child.stderr?.on('data', append);
    child.on('error', () => {
      reason = 'Executable unavailable or failed to spawn';
    });
    child.on('exit', () => {
      terminate();
    });
    child.on('close', (exitCode, exitSignal) => {
      clearTimeout(timer);
      clearTimeout(killTimer);
      const outcome = owned.finish(exitCode, exitSignal);
      if (outcome.error) reason = `${reason ?? 'Process finished'}; ${outcome.error}`;
      signal.removeEventListener('abort', cancel);
      resolve({ exitCode: outcome.code, signal: outcome.signal, reason, output, bytes });
    });
  });
}
