import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: '.',
  testMatch: '*.spec.ts',
  workers: 1,
  fullyParallel: false,
  timeout: 30_000,
  retries: 0,
  outputDir: '../test-results',
  reporter: [['list'], ['json', { outputFile: '../test-results/results.json' }]],
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
