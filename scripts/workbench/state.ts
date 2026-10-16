import type { ProjectSnapshot, SnapshotResponse } from './contracts/index.ts';
import { collectSnapshot } from './snapshot.ts';
export class SnapshotUnavailable extends Error {
  constructor() {
    super('Snapshot unavailable: inspect repository bounds and refresh again.');
  }
}
export class SnapshotState {
  private snapshot: ProjectSnapshot | null = null;
  private stale = false;
  private pending: Promise<void> | null = null;
  private closed = false;
  private root: string;
  constructor(root: string) {
    this.root = root;
  }
  current(): SnapshotResponse {
    if (!this.snapshot) throw new SnapshotUnavailable();
    return {
      snapshot: this.snapshot,
      stale: this.stale,
      diagnostics: this.stale
        ? [
            {
              severity: 'error',
              code: 'refresh-failed',
              message: 'Refresh failed or source changed; previous snapshot retained.',
            },
          ]
        : [],
    };
  }
  async refresh() {
    if (this.closed) throw new Error('Service closing');
    if (!this.pending)
      this.pending = collectSnapshot(this.root)
        .then((snapshot) => {
          if (!this.closed) {
            this.snapshot = snapshot;
            this.stale = false;
          }
        })
        .catch(() => {
          this.stale = true;
        })
        .finally(() => {
          this.pending = null;
        });
    await this.pending;
    return this.current();
  }
  async close() {
    this.closed = true;
    await this.pending;
  }
}
