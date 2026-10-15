export interface Source {
  revision: string;
  dirtyDiffSha256: string | null;
}
export interface DiscoveredTest {
  id: string;
  runner: string;
  level: string;
  platform: string;
  pointerWidth?: number;
  binarySha256: string;
  testBinarySha256?: string;
}
export interface Result {
  testId: string;
  attempt: number;
  status: string;
  durationMs: number;
  timestamp: string;
  binarySha256: string;
  testBinarySha256?: string;
  platform: string;
  toolchain: string;
  device: string;
  environment: { valid: boolean; reasons: string[] };
  artifacts: string[];
  output?: string;
  os?: string;
  inputRoute?: string;
}
export interface Coverage {
  automation: string;
  testIds: string[];
  gaps: { reason: string; ownerTask: string }[];
}
export interface EnumerationScope {
  runner: string;
  platform: string;
  level: string;
}
export interface Requirement {
  coverage?: Coverage;
  platform: string;
  level: string;
  pointerWidth?: number;
}
export interface Case {
  id: string;
  expected: string;
  invariants: string[];
  ownerTask: string;
  requirements: Requirement[];
  testIds: string[];
  automation: string;
  gaps: { reason: string; ownerTask: string }[];
}
export interface Feature {
  schemaVersion: number;
  id: string;
  description: string;
  sourceRevision: string;
  cases: Case[];
}
export interface Discovery {
  scopes?: EnumerationScope[];
  schemaVersion: number;
  source: Source;
  tests: DiscoveredTest[];
}
export interface Execution {
  schemaVersion: number;
  source: Source;
  results: Result[];
}
