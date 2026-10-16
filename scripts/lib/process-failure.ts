/** Structured termination evidence; messages remain useful in durable command logs. */
export class ProcessFailure extends Error {
  readonly exitCode: number | null;
  readonly signal: NodeJS.Signals | null;
  readonly reason: string;
  readonly output: string;

  constructor(
    executable: string,
    code: number | null,
    signal: NodeJS.Signals | null,
    reason: string,
    output: string,
  ) {
    super(`${executable}: ${reason || `exit ${code}, signal ${signal}`}\n${output}`);
    this.exitCode = code;
    this.signal = signal;
    this.reason = reason;
    this.output = output;
  }
}
