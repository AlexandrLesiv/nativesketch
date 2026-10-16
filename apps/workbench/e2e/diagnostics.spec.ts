import { test, expect } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { snapshotFixture } from './fixture.ts';
import { offlineFile } from './offline.ts';

test('DX-COVERAGE-001 invalid requirements are unavailable rather than zero or filtered out', async ({
  page,
  context,
}) => {
  const directory = await mkdtemp(join(tmpdir(), 'workbench-invalid-'));
  try {
    const snapshot = snapshotFixture();
    snapshot.coverage.state = 'invalid';
    snapshot.coverage.data.rows = [];
    const file = join(directory, 'report.html');
    await offlineFile(file, snapshot);
    await context.setOffline(true);
    await page.goto(pathToFileURL(file).href);
    const card = page
      .locator('.card')
      .filter({ has: page.getByRole('heading', { name: 'Registered requirements loaded' }) });
    await expect(card).toContainText('Unavailable — validation failed');
    await expect(card.locator('.count')).toHaveCount(0);
    await page.getByRole('link', { name: 'Coverage', exact: true }).click();
    await expect(page.getByRole('main')).toContainText(
      'Requirements unavailable because coverage validation failed.',
    );
    await expect(page.getByRole('main')).not.toContainText('No requirements match these filters.');
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test('DX-DEP-001 dependency inventory stays concise and preserves every relationship in details', async ({
  page,
  context,
}) => {
  const directory = await mkdtemp(join(tmpdir(), 'workbench-dependencies-'));
  try {
    const snapshot = snapshotFixture();
    const dependency = snapshot.dependencies.data[0];
    if (!dependency) throw new Error('Dependency fixture missing');
    dependency.dependencies = Array.from(
      { length: 30 },
      (_, index) => `registry+fixture://package-${index}@1.0.0`,
    );
    const file = join(directory, 'report.html');
    await offlineFile(file, snapshot);
    await context.setOffline(true);
    await page.goto(pathToFileURL(file).href);
    await page.getByRole('link', { name: 'Dependencies', exact: true }).click();
    await expect(page.getByRole('table')).toContainText('30 resolved; select package for details');
    await expect(page.getByRole('table')).not.toContainText('registry+fixture');
    await page.getByRole('link', { name: 'skia-safe', exact: true }).click();
    await expect(page.locator('#selected-detail')).toBeFocused();
    for (const id of dependency.dependencies)
      await expect(page.getByRole('main')).toContainText(id);
    await expect(page.getByRole('main')).toContainText('Provenance: Cargo.toml');
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
