import { defineConfig } from '@playwright/test';
const output = process.env.SKETCH_BROWSER_OUTPUT;
const executablePath = process.env.SKETCH_BROWSER_EXECUTABLE;
const baseURL = process.env.SKETCH_BROWSER_URL;
if (!output || !executablePath || !baseURL) throw new Error('Use scripts/test-web-e2e.ts');
export default defineConfig({
  testDir: '.',
  testMatch: '*.spec.ts',
  timeout: 30_000,
  expect: { timeout: 8_000 },
  workers: 1,
  retries: 0,
  fullyParallel: false,
  forbidOnly: true,
  outputDir: `${output}/test-artifacts`,
  reporter: [['line'], ['json', { outputFile: process.env.SKETCH_BROWSER_REPORT }]],
  projects: [{ name: 'chromium', use: { browserName: 'chromium' } }],
  use: {
    baseURL,
    headless: true,
    viewport: { width: 1100, height: 760 },
    launchOptions: { executablePath },
    acceptDownloads: true,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    video: 'off',
  },
});
