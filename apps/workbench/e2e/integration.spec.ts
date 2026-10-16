import { test, expect } from '@playwright/test';
import { join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { readFile } from 'node:fs/promises';
import { startWorkbench } from '../../../scripts/workbench/index.ts';
import type { SessionResponse } from '../../../scripts/workbench/contracts/index.ts';
const root = fileURLToPath(new URL('../../../', import.meta.url));
// Actual session capabilities must never be persisted in Playwright network traces.
test.use({ trace: 'off' });

test('DX-EXPORT-001 actual service exports the real project for offline browsing', async ({
  page,
  context,
  request,
}, info) => {
  test.setTimeout(120_000);
  const service = await startWorkbench({ root, assets: join(root, 'apps/workbench/dist') });
  try {
    await page.goto(service.url);
    await expect(
      page.getByRole('heading', { name: 'Project workbench', exact: true }),
    ).toBeVisible();
    await expect(page.getByRole('link', { name: 'Tasks', exact: true })).toBeVisible();
    for (const view of [
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
        .getByRole('link', { name: view, exact: true })
        .click();
      await page.screenshot({
        path: info.outputPath(`actual-${view.replaceAll(/[^a-z]/gi, '-')}.png`),
        fullPage: false,
      });
    }
    await page.getByRole('link', { name: 'Actions & jobs', exact: true }).click();
    await expect(
      page.getByRole('button', { name: 'Run Run macOS app', exact: true }),
    ).toBeDisabled();
    await page.getByRole('button', { name: 'Run Export offline report', exact: true }).click();
    await expect(
      page.getByRole('heading', { name: 'export-report · passed', exact: true }),
    ).toBeVisible({ timeout: 60_000 });
    const downloaded = page.waitForEvent('download');
    await page.getByRole('button', { name: /Download artifact .*\/report\.html$/ }).click();
    const report = info.outputPath('actual-offline-report.html');
    await (await downloaded).saveAs(report);
    const session = (await (
      await request.get(service.url + 'api/session', {
        headers: { 'Sec-Fetch-Site': 'same-origin' },
      })
    ).json()) as SessionResponse;
    expect(await readFile(report, 'utf8')).not.toContain(session.capability);
    await context.setOffline(true);
    await page.goto(pathToFileURL(report).href);
    await expect(page.getByText(/Static report · read-only/)).toBeVisible();
    await page.getByRole('link', { name: 'Docs', exact: true }).click();
    await page
      .getByRole('link', { name: /docs\/architecture\.md$/ })
      .first()
      .click();
    await expect(page.locator('#selected-detail')).toBeFocused();
    await expect(page.getByRole('button', { name: 'Refresh snapshot' })).toBeDisabled();
  } finally {
    await context.setOffline(false);
    await service.close();
  }
});

test('DX-PREVIEW-001 actual Wasm build runs on an isolated origin without admin access', async ({
  page,
  context,
  request,
}) => {
  test.setTimeout(180_000);
  const service = await startWorkbench({ root, assets: join(root, 'apps/workbench/dist') });
  try {
    await page.goto(service.url);
    await page.getByRole('link', { name: 'Actions & jobs', exact: true }).click();
    await page
      .getByRole('button', { name: 'Run Build isolated Wasm preview', exact: true })
      .click();
    await expect(
      page.getByRole('heading', { name: 'preview-web · passed', exact: true }),
    ).toBeVisible({ timeout: 150_000 });
    const href = await page
      .getByRole('link', { name: 'Open isolated Canvas2D preview' })
      .getAttribute('href');
    if (!href) throw new Error('Preview URL missing');
    expect(new URL(href).origin).not.toBe(new URL(service.url).origin);
    const drawing = await context.newPage();
    await drawing.goto(href);
    const canvas = drawing.locator('#canvas');
    await expect(drawing.locator('#status')).not.toContainText('Loading drawing engine', {
      timeout: 15_000,
    });
    const bounds = await canvas.boundingBox();
    if (!bounds) throw new Error('Actual drawing canvas absent');
    await drawing.mouse.move(bounds.x + 60, bounds.y + 60);
    await drawing.mouse.down();
    await drawing.mouse.move(bounds.x + 160, bounds.y + 110, { steps: 10 });
    await drawing.mouse.up();
    await expect(drawing.getByRole('button', { name: 'Undo', exact: true })).toBeEnabled();
    const response = await request.get(service.url + 'api/session', {
      headers: { Origin: new URL(href).origin, 'Sec-Fetch-Site': 'same-site' },
    });
    expect(response.status()).toBe(403);
    const session = (await (
      await request.get(service.url + 'api/session', {
        headers: { 'Sec-Fetch-Site': 'same-origin' },
      })
    ).json()) as SessionResponse;
    expect(await drawing.content()).not.toContain(session.capability);
    await drawing.close();
  } finally {
    await service.close();
  }
});
