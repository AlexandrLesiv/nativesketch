import { OwnershipFailure } from './ownership-diagnostics.ts';
import type { OwnershipDiagnostic } from './ownership-diagnostics.ts';
/** Signal every registered group; a transient error is not proof of successful cleanup. */
export function signalGroups(groups: readonly { pid: number }[], signal: NodeJS.Signals) {
  const diagnostics: OwnershipDiagnostic[] = [];
  for (const item of groups) {
    try {
      process.kill(-item.pid, signal);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ESRCH')
        diagnostics.push(new OwnershipFailure(item.pid, 'signal', signal, error).diagnostic);
    }
  }
  return diagnostics;
}
/** Darwin may return EPERM for an all-zombie group. Only eventual ESRCH certifies exit. */
export function confirmGroupsExited(groups: readonly { pid: number }[]) {
  let remaining = [...groups];
  const diagnostics = new Map<number, OwnershipDiagnostic>();
  for (let attempt = 0; attempt < 200 && remaining.length; attempt++) {
    remaining = remaining.filter((item) => {
      try {
        process.kill(-item.pid, 0);
        return true;
      } catch (error) {
        const code = (error as NodeJS.ErrnoException).code;
        if (code === 'ESRCH') return false;
        const failure = new OwnershipFailure(item.pid, 'exit-probe', '0', error);
        if (code !== 'EPERM') throw failure;
        if (!diagnostics.has(item.pid)) diagnostics.set(item.pid, failure.diagnostic);
        return true;
      }
    });
    if (remaining.length) Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 5);
  }
  if (remaining.length)
    throw new Error(
      `Owned groups did not finish within cleanup bound: ${remaining.map((item) => item.pid).join(',')}; diagnostics=${JSON.stringify([...diagnostics.values()])}`,
    );
  return [...diagnostics.values()];
}
