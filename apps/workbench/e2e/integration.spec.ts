import { test, expect } from '@playwright/test';
import { join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { readFile } from 'node:fs/promises';
import { startWorkbench } from '../../../scripts/workbench/index.ts';
import type {
  SessionResponse,
  PreviewResponse,
  JobsResponse,
} from '../../../scripts/workbench/contracts/index.ts';
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
      await expect(
        page.getByRole('main').getByRole('heading', { name: view, exact: true }),
      ).toBeVisible();
      await expect(
        page
          .getByRole('navigation', { name: 'Workbench destinations' })
          .getByRole('link', { name: view, exact: true }),
      ).toHaveAttribute('aria-current', 'page');
      if (view === 'Coverage')
        await expect(page.getByRole('table').getByRole('row')).not.toHaveCount(1);
      await page.evaluate(() => {
        window.scrollTo(0, 0);
      });
      const capture = info.outputPath(`actual-${view.replaceAll(/[^a-z]/gi, '-')}.png`);
      await page.screenshot({
        path: capture,
        fullPage: false,
      });
      await info.attach(`actual-${view}`, { path: capture, contentType: 'image/png' });
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
    expect((await readFile(report, 'utf8')).includes(session.capability)).toBe(false);
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
}, info) => {
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
      .getByRole('link', { name: 'Open isolated CanvasKit 0.42.0 preview' })
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
    const headers = {
      'Sec-Fetch-Site': 'same-origin',
      'X-Workbench-Capability': session.capability,
    };
    const preview = (await (
      await request.get(service.url + 'api/preview', { headers })
    ).json()) as PreviewResponse;
    const jobs = (await (
      await request.get(service.url + 'api/jobs', { headers })
    ).json()) as JobsResponse;
    const job = jobs.jobs.find(
      (item) => item.actionId === 'preview-web' && item.state === 'passed',
    );
    expect(preview.preview?.sha256).toMatch(/^[a-f0-9]{64}$/);
    expect(preview.preview?.source).toEqual(job?.source);
    expect(job?.sourceChanged).toBe(false);
    await info.attach('actual-preview-identity', {
      body: JSON.stringify({ preview: preview.preview, job }, null, 2),
      contentType: 'application/json',
    });
    const capture = info.outputPath('actual-wasm-drawing.png');
    await drawing.screenshot({ path: capture });
    await info.attach('actual-wasm-drawing', { path: capture, contentType: 'image/png' });
    expect((await drawing.content()).includes(session.capability)).toBe(false);
    await drawing.close();
  } finally {
    await service.close();
  }
});
