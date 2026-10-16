import { test, expect } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { snapshotFixture } from './fixture.ts';
import { offlineFile } from './offline.ts';
let directory: string;
let url: string;
test.beforeAll(async () => {
  directory = await mkdtemp(join(tmpdir(), 'workbench-ui-'));
  const file = join(directory, 'report.html');
  await offlineFile(file, snapshotFixture());
  url = pathToFileURL(file).href;
});
test.afterAll(async () => {
  await rm(directory, { recursive: true, force: true });
});
test.beforeEach(async ({ context, page }) => {
  await context.setOffline(true);
  await page.goto(url);
});
test('DX-NAV-001 seven views, URL filters and Back retain selection', async ({ page }) => {
  for (const name of [
    'Overview',
    'Tasks',
    'Docs',
    'Dependencies',
    'Coverage',
    'Actions & jobs',
    'Metrics',
  ]) {
    await page
      .getByRole('navigation', { name: 'Workbench destinations' })
      .getByRole('link', { name, exact: true })
      .click();
    await expect(page.getByRole('main').getByRole('heading', { name, exact: true })).toBeVisible();
  }
  await page.getByRole('link', { name: 'Tasks', exact: true }).click();
  await page.getByRole('searchbox', { name: 'Search tasks' }).fill('Core');
  await page.getByRole('button', { name: 'Apply filters' }).click();
  await page.getByRole('table').getByRole('link', { name: '001 Core commands' }).click();
  await expect(page.getByRole('heading', { name: '001 Core commands' })).toBeVisible();
  await expect(page.locator('#selected-detail')).toBeFocused();
  await page.getByRole('link', { name: 'Docs', exact: true }).click();
  await page.goBack();
  await expect(page.getByRole('searchbox')).toHaveValue('Core');
  await expect(page.getByRole('heading', { name: '001 Core commands' })).toBeVisible();
});
test('DX-DOC-001 offline document links and literal hostile content', async ({ page }) => {
  await page.getByRole('link', { name: 'Docs', exact: true }).click();
  await page.getByRole('link', { name: 'Architecture — docs/architecture.md' }).click();
  await expect(page.locator('article')).toContainText('<script>window.pwned=true</script>');
  await expect(page.locator('article')).toContainText('Not in this snapshot');
  await expect(page.locator('article img, article script')).toHaveCount(0);
  expect(await page.evaluate(() => Object.hasOwn(window, 'pwned'))).toBe(false);
  await page.getByRole('link', { name: 'Named heading', exact: true }).click();
  await expect(page).toHaveURL(/heading=architecture/);
  await expect(page.locator('#doc-heading-architecture')).toBeVisible();
  await page.getByRole('link', { name: 'Task', exact: true }).click();
  await expect(
    page.getByRole('main').getByRole('heading', { name: 'Core task', exact: true }).first(),
  ).toBeVisible();
});
test('DX-COVERAGE-001 stale attempts and missing metrics stay distinct', async ({ page }) => {
  await page.getByRole('link', { name: 'Coverage', exact: true }).click();
  await expect(page.getByRole('main')).toContainText('Evidence source freshness: stale');
  await page.getByRole('link', { name: 'DX-FIXTURE · fixture' }).click();
  await expect(
    page.getByText('fixture-test · attempt 1 · failed · environment invalid', { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText('fixture-test · attempt 2 · passed · environment valid', { exact: true }),
  ).toBeVisible();
  await page.getByRole('link', { name: 'Metrics', exact: true }).click();
  await expect(page.getByRole('table')).toContainText('Unavailable');
  await expect(page.getByRole('table')).toContainText('missing');
});
test('DX-EXPORT-001 static report never requests network or enables execution', async ({
  page,
}) => {
  const requests: string[] = [];
  page.on('request', (request) => {
    if (/^https?:/.test(request.url())) requests.push(request.url());
  });
  await page.getByRole('link', { name: 'Actions & jobs', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Run Fixture background check' })).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Run Native run' })).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Refresh snapshot' })).toBeDisabled();
  expect(requests).toEqual([]);
});
test('DX-ACCESS-001 keyboard graph and 320px reflow screenshots', async ({ page }, info) => {
  await page.getByRole('link', { name: 'Tasks', exact: true }).click();
  await page.getByText('Show dependency graph', { exact: true }).click();
  const graphLink = page
    .getByRole('group', { name: 'Task dependencies' })
    .getByRole('link', { name: '002 Native adapter: waiting' });
  await graphLink.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { name: '002 Native adapter' })).toBeVisible();
  await page.screenshot({ path: info.outputPath('desktop-light.png'), fullPage: true });
  await page.setViewportSize({ width: 320, height: 800 });
  await expect
    .poll(() => page.evaluate(() => document.documentElement.scrollWidth <= innerWidth))
    .toBe(true);
  await page.emulateMedia({ colorScheme: 'dark', reducedMotion: 'reduce' });
  await page.screenshot({ path: info.outputPath('mobile-dark.png'), fullPage: true });
});
