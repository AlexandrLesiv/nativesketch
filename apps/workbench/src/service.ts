import type {
  Job,
  JobsResponse,
  JobResponse,
  PreviewResponse,
  SessionResponse,
  SnapshotResponse,
} from '../../../scripts/workbench/contracts/index.ts';
export class LocalService {
  #capability = '';
  connected = false;
  async request<T>(path: string, body?: object): Promise<T> {
    const response = await fetch(path, {
      method: body ? 'POST' : 'GET',
      credentials: 'same-origin',
      cache: 'no-store',
      headers: {
        ...(this.#capability ? { 'X-Workbench-Capability': this.#capability } : {}),
        ...(body ? { 'Content-Type': 'application/json' } : {}),
      },
      ...(body ? { body: JSON.stringify(body) } : {}),
      signal: AbortSignal.timeout(15_000),
    });
    if (!response.ok)
      throw new Error(
        `Service rejected ${path}: HTTP ${response.status} — ${(await response.text()).slice(0, 2000)}`,
      );
    return (await response.json()) as T;
  }
  async connect(): Promise<void> {
    const session = await this.request<SessionResponse>('/api/session');
    if (!session.capability || session.policy.foregroundExecution !== false)
      throw new Error('Unsupported service policy');
    this.#capability = session.capability;
    this.connected = true;
  }
  async artifact(id: string): Promise<void> {
    const response = await fetch('/api/artifacts/' + encodeURIComponent(id), {
      headers: { 'X-Workbench-Capability': this.#capability },
      credentials: 'same-origin',
      signal: AbortSignal.timeout(15_000),
    });
    if (!response.ok) throw new Error('Artifact unavailable: HTTP ' + response.status);
    const blob = await response.blob();
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = id.split('/').at(-1) || 'evidence.txt';
    anchor.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  snapshot(refresh = false): Promise<SnapshotResponse> {
    return this.request('/api/' + (refresh ? 'refresh' : 'snapshot'), refresh ? {} : undefined);
  }
  async jobs(): Promise<Job[]> {
    return (await this.request<JobsResponse>('/api/jobs')).jobs;
  }
  preview(): Promise<PreviewResponse> {
    return this.request('/api/preview');
  }
  start(actionId: string): Promise<JobResponse> {
    return this.request('/api/jobs', { actionId });
  }
  cancel(id: string): Promise<JobResponse> {
    return this.request(`/api/jobs/${encodeURIComponent(id)}/cancel`, {});
  }
}
