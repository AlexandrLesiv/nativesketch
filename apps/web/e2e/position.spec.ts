import { test, expect } from '@playwright/test';
import { ready, stroke } from './helpers.ts';
import { screenshotInk } from './screenshot-pixels.ts';
test('WEB-POSITION-001 stroke pixels stay at the actual input coordinates', async ({
  page,
}, info) => {
  await ready(page);
  await stroke(page);
  const ink = await screenshotInk(page);
  expect(ink.count).toBeGreaterThan(100);
  expect(ink.left).toBeGreaterThanOrEqual(96);
  expect(ink.left).toBeLessThanOrEqual(100);
  expect(ink.top).toBeGreaterThanOrEqual(96);
  expect(ink.top).toBeLessThanOrEqual(100);
  expect(ink.right).toBeGreaterThanOrEqual(259);
  expect(ink.right).toBeLessThanOrEqual(264);
  expect(ink.bottom).toBeGreaterThanOrEqual(159);
  expect(ink.bottom).toBeLessThanOrEqual(164);
  const path = info.outputPath('stroke-position.png');
  await page.locator('#canvas').screenshot({ path });
  await info.attach('stroke-position', { path, contentType: 'image/png' });
});
