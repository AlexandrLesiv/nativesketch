import { randomUUID } from 'node:crypto';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import type { Job } from './contracts/index.ts';
import { sourceIdentity } from './identity.ts';
import { actionCatalog, actionArguments } from './actions.ts';
import { safeText, sanitize } from './files.ts';
import { storageDirectory } from './storage.ts';
import { runOwned } from './process.ts';
export class Jobs {
  private records: Job[] = [];
  private active: { id: string; controller: AbortController; done: Promise<void> } | null = null;
  private closed = false;
  constructor(privateRoot: string, completed?: (job: Job, signal: AbortSignal) => Promise<void>) {
    this.root = privateRoot;
    this.completed = completed;
  }
  private root: string;
  private completed: ((job: Job, signal: AbortSignal) => Promise<void>) | undefined;
  list() {
    return sanitize(this.records);
  }
  async enqueue(actionId: string) {
    if (this.closed || this.records.filter((j) => j.state === 'queued').length >= 4)
      throw new Error('Queue unavailable or full');
    const spec = (await actionCatalog()).find((a) => a.id === actionId);
    if (!spec?.enabled || spec.foreground)
      throw new Error(spec?.disabledReason ?? 'Unknown action');
    const job: Job = {
      id: randomUUID(),
      actionId,
      state: 'queued',
      source: await sourceIdentity(this.root),
      queuedAt: new Date().toISOString(),
      startedAt: null,
      endedAt: null,
      commandPreview: spec.commandPreview,
      exitCode: null,
      signal: null,
      reason: null,
      log: '',
      outputBytes: 0,
      sourceChanged: false,
      artifactIds: [],
    };
    while (this.records.length >= 30) {
      const i = this.records.findIndex(
        (j) => !['queued', 'running', 'cancelling'].includes(j.state),
      );
      if (i < 0) throw new Error('Job history full');
      this.records.splice(i, 1);
    }
    if (this.closed || this.records.filter((j) => j.state === 'queued').length >= 4)
      throw new Error('Queue unavailable or full');
    this.records.push(job);
    this.pump();
    return sanitize(job);
  }
  cancel(id: string) {
    const job = this.records.find((j) => j.id === id);
    if (!job) throw new Error('Unknown job');
    if (job.state === 'queued') {
      if (this.active?.id === id) this.active.controller.abort();
      job.state = 'cancelled';
      job.endedAt = new Date().toISOString();
      job.reason = 'Cancelled before start';
    }
    if (job.state === 'running') {
      job.state = 'cancelling';
      this.active?.controller.abort();
    }
    return sanitize(job);
  }
  async close() {
    this.closed = true;
    for (const job of this.records)
      if (job.state === 'queued' || job.state === 'running') this.cancel(job.id);
    await this.active?.done;
  }
  private pump() {
    if (this.active || this.closed) return;
    const job = this.records.find((j) => j.state === 'queued');
    if (!job) return;
    const controller = new AbortController();
    const done = this.execute(job, controller).finally(() => {
      this.active = null;
      this.pump();
    });
    this.active = { id: job.id, controller, done };
  }
  private async execute(job: Job, controller: AbortController) {
    try {
      const spec = (await actionCatalog()).find((a) => a.id === job.actionId);
      if (!spec?.enabled || spec.foreground) throw new Error('Action no longer eligible');
      if (JSON.stringify(job.source) !== JSON.stringify(await sourceIdentity(this.root)))
        throw new Error('Source changed before dequeue');
      if (controller.signal.aborted || job.state === 'cancelled') return;
      const artifactPath = `artifacts/workbench/jobs/${job.id}`;
      await storageDirectory(this.root, artifactPath);
      if (controller.signal.aborted) return;
      const args = actionArguments(job.actionId, artifactPath);
      job.commandPreview = `node ${args.join(' ')}`;
      job.startedAt = new Date().toISOString();
      job.state = 'running';
      const result = await runOwned(
        process.execPath,
        args,
        this.root,
        controller.signal,
        (log, bytes) => {
          job.log = safeText(log);
          job.outputBytes = bytes;
        },
      );
      job.exitCode = result.exitCode;
      job.signal = result.signal;
      job.reason = result.reason;
      const succeeded = result.exitCode === 0 && !result.reason && !result.signal;
      await writeFile(join(this.root, artifactPath, 'output.txt'), job.log);
      job.artifactIds.push(`${artifactPath}/output.txt`);
      job.sourceChanged =
        JSON.stringify(job.source) !== JSON.stringify(await sourceIdentity(this.root));
      if (succeeded && !job.sourceChanged && !controller.signal.aborted)
        await this.completed?.(job, controller.signal);
      job.sourceChanged ||=
        JSON.stringify(job.source) !== JSON.stringify(await sourceIdentity(this.root));
      if (controller.signal.aborted) {
        job.state = 'cancelled';
        job.reason = 'Cancelled';
      } else if (job.sourceChanged) {
        job.state = 'failed';
        job.reason = 'Source changed during execution; artifacts cannot certify original source';
      } else job.state = succeeded ? 'passed' : 'failed';
      if (job.actionId === 'export-report' && job.state === 'passed')
        job.artifactIds.push(`${artifactPath}/report.html`);
    } catch (error) {
      job.state = controller.signal.aborted ? 'cancelled' : 'failed';
      job.reason = controller.signal.aborted
        ? 'Cancelled'
        : error instanceof Error
          ? safeText(error.message)
          : 'Job failed';
    } finally {
      job.endedAt = new Date().toISOString();
    }
  }
}
