import type { JSONReportSpec } from 'playwright/types/testReporter';
export function spec(): JSONReportSpec {
  return {
    tags: [],
    title: 'WEB-HISTORY-001 history',
    ok: true,
    id: 'one',
    file: 'editing.spec.ts',
    line: 1,
    column: 1,
    tests: [
      {
        timeout: 30000,
        annotations: [],
        expectedStatus: 'passed',
        projectName: 'chromium',
        projectId: 'chromium',
        status: 'expected',
        results: [
          {
            workerIndex: 0,
            parallelIndex: 0,
            status: 'passed',
            duration: 10,
            error: undefined,
            errors: [],
            stdout: [],
            stderr: [],
            retry: 0,
            startTime: '2026-10-16T00:00:00Z',
            attachments: [],
            annotations: [],
          },
        ],
      },
    ],
  };
}
