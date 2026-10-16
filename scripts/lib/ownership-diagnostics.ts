export interface OwnershipDiagnostic {
  timestamp: string;
  pid: number;
  phase: 'signal' | 'exit-probe';
  signal: string;
  code: string;
}
export class OwnershipFailure extends Error {
  readonly diagnostic: OwnershipDiagnostic;
  constructor(pid: number, phase: 'signal' | 'exit-probe', signal: string, error: unknown) {
    const code = (error as NodeJS.ErrnoException).code ?? 'UNKNOWN';
    super(`Owned group ${pid} ${phase} ${signal}: ${code}`);
    this.diagnostic = { timestamp: new Date().toISOString(), pid, phase, signal, code };
  }
}
