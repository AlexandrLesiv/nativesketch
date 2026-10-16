import { test, expect } from '@playwright/test';
import { fixtureServer } from './fixture-server.ts';
let fixture: Awaited<ReturnType<typeof fixtureServer>>;
test.beforeAll(async () => {
  fixture = await fixtureServer();
});
test.afterAll(async () => {
  await fixture.close();
});
test('DX-JOB-001 fixture protocol failure, retry and cancel preserve history', async ({ page }) => {
  await page.goto(fixture.url);
  await page.getByRole('link', { name: 'Actions & jobs', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Run Native run' })).toBeDisabled();
  await page.getByRole('button', { name: 'Run Fixture background check' }).click();
  await expect(page.getByRole('heading', { name: 'fixture-check · failed' })).toBeVisible();
  await expect(page.getByText('Deliberate fixture failure')).toBeVisible();
  await page.getByRole('button', { name: 'Run Fixture background check' }).click();
  await expect(page.getByRole('heading', { name: 'fixture-check · running' })).toBeVisible();
  await expect(page.locator('[data-job="fixture-1"]')).toBeFocused();
  const running = page.locator('[data-job="fixture-1"]');
  await running.getByText(/Output \(/).click();
  await running.getByRole('button', { name: 'Cancel fixture-1' }).focus();
  await expect
    .poll(() => running.locator('pre').last().textContent(), { timeout: 10_000 })
    .toContain('Fixture heartbeat\nFixture heartbeat');
  await expect(running.locator('details').last()).toHaveAttribute('open', '');
  await expect(running.getByRole('button', { name: 'Cancel fixture-1' })).toBeFocused();
  const download = page.waitForEvent('download');
  await running.getByRole('button', { name: 'Download artifact fixture.txt' }).click();
  expect((await download).suggestedFilename()).toBe('fixture.txt');
  await page.getByRole('button', { name: 'Cancel fixture-1' }).click();
  await expect(page.getByRole('heading', { name: 'fixture-check · cancelled' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'fixture-check · failed' })).toBeVisible();
  await expect(page.locator('[data-job] script')).toHaveCount(0);
});
test('DX-SOURCE-001 live snapshot has exact source, stale label and read-only foreground policy', async ({
  page,
}) => {
  await page.goto(fixture.url);
  await expect(page.getByRole('status')).toContainText('Snapshot is stale');
  await page.locator('header summary').click();
  await expect(page.locator('header')).toContainText('fixture-revision-1234567890');
  await expect(page.locator('header')).toContainText('fixture-dirty-hash');
  await page.getByRole('link', { name: 'Actions & jobs', exact: true }).click();
  await expect(page.getByRole('main')).toContainText('Foreground execution is disabled');
});
