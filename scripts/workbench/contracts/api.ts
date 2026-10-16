import type { ProjectSnapshot, Source, Diagnostic } from './snapshot.ts';
export interface ServicePolicy {
  foregroundExecution: false;
  maximumQueuedJobs: 4;
  maximumRetainedJobs: 30;
}
export interface SessionResponse {
  capability: string;
  policy: ServicePolicy;
}
export interface ApiError {
  error: { code: string; message: string };
  diagnostics?: Diagnostic[];
}
export interface SnapshotResponse {
  snapshot: ProjectSnapshot;
  stale: boolean;
  diagnostics: Diagnostic[];
}
export type JobState =
  | 'queued'
  | 'running'
  | 'cancelling'
  | 'passed'
  | 'failed'
  | 'cancelled'
  | 'blocked';
export interface Job {
  id: string;
  actionId: string;
  state: JobState;
  source: Source;
  queuedAt: string;
  startedAt: string | null;
  endedAt: string | null;
  commandPreview: string;
  exitCode: number | null;
  signal: string | null;
  reason: string | null;
  log: string;
  outputBytes: number;
  sourceChanged: boolean;
  artifactIds: string[];
}
export interface JobsResponse {
  jobs: Job[];
}
export interface JobRequest {
  actionId: string;
}
export interface JobResponse {
  job: Job;
}
export interface Preview {
  source: Source;
  sha256: string;
  url: string;
  builtAt: string;
  renderer: 'CanvasKit 0.42.0';
}
export interface PreviewResponse {
  preview: Preview | null;
  reason: string | null;
}
/** JSON in script#workbench-bootstrap; service bootstrap has no token or repository data. */
export type Bootstrap = { mode: 'live' } | { mode: 'static'; snapshot: ProjectSnapshot };
/** Every JSON call except session sends X-Workbench-Capability. Mutations also require Origin. */
export interface EndpointPayloads {
  'GET /api/session': SessionResponse;
  'GET /api/snapshot': SnapshotResponse;
  'POST /api/refresh': SnapshotResponse;
  'GET /api/jobs': JobsResponse;
  'POST /api/jobs': JobResponse;
  'POST /api/jobs/{id}/cancel': JobResponse;
  'GET /api/preview': PreviewResponse;
}
