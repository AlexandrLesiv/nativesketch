import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: '.',
  testMatch: '*.spec.ts',
  workers: 1,
  fullyParallel: false,
  timeout: 30_000,
  retries: 0,
  forbidOnly: true,
  outputDir: process.env.SKETCH_WORKBENCH_OUTPUT ?? '../test-results',
  reporter: [
    ['list'],
    [
      'json',
      {
        outputFile: process.env.SKETCH_WORKBENCH_REPORT ?? '../test-results/results.json',
      },
    ],
  ],
  projects: [{ name: 'chromium', use: { browserName: 'chromium' } }],
  use: {
    headless: true,
    viewport: { width: 1280, height: 900 },
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    launchOptions: process.env.SKETCH_BROWSER_EXECUTABLE
      ? { executablePath: process.env.SKETCH_BROWSER_EXECUTABLE }
      : {},
  },
});
