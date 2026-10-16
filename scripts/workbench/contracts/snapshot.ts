import type { Source, Result } from '../../evidence/types.ts';
export type { Source } from '../../evidence/types.ts';
export interface Diagnostic {
  severity: 'info' | 'warning' | 'error';
  code: string;
  path?: string;
  message: string;
}
export interface Section<T> {
  state: 'ready' | 'partial' | 'missing' | 'invalid';
  diagnostics: Diagnostic[];
  data: T;
}
export interface Task {
  id: string;
  title: string;
  status: string;
  brief: string;
  depends_on: string[];
  roles: string[];
}
export interface Document {
  id: string;
  title: string;
  format: 'markdown' | 'text';
  text: string;
  sha256: string;
}
export interface Dependency {
  id: string;
  ecosystem: string;
  name: string;
  version: string | null;
  versionKind: 'resolved' | 'declared' | 'unknown';
  scope: 'direct' | 'transitive' | 'unknown';
  license: string | null;
  provenance: string;
  dependencies: string[];
}
export interface CoverageRow {
  feature: string;
  caseId: string;
  expected: string;
  platform: string;
  level: string;
  pointerWidth?: number;
  automation: string;
  status: string;
  testIds: string[];
  discoveredTestIds: string[];
  gaps: { reason: string; ownerTask: string }[];
  attempts: Result[];
  ownerTask: string;
  invariants: string[];
}
export interface CoverageData {
  rows: CoverageRow[];
  issues: string[];
  discoverySource: Source | null;
  freshness: 'current' | 'stale' | 'unknown';
  evidencePaths: string[];
}
export interface Metric {
  id: string;
  label: string;
  value: number | null;
  unit: string;
  status: 'measured' | 'missing' | 'invalid';
  source: Source | null;
  capturedAt: string | null;
  scope: string;
  configuration: string;
  evidencePaths: string[];
}
export interface ActionSpec {
  id: string;
  label: string;
  category: string;
  platform: string;
  commandPreview: string;
  foreground: boolean;
  enabled: boolean;
  disabledReason: string | null;
  prerequisites: string[];
}
export interface ProjectSnapshot {
  schemaVersion: 1;
  generatedAt: string;
  source: Source;
  branch: string;
  host: { platform: string; architecture: string };
  tasks: Section<Task[]>;
  documents: Section<Document[]>;
  dependencies: Section<Dependency[]>;
  coverage: Section<CoverageData>;
  metrics: Section<Metric[]>;
  actions: Section<ActionSpec[]>;
}
