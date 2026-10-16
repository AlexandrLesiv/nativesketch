import { test } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { snapshotFixture } from './fixture.ts';
import { offlineFile } from './offline.ts';
test('DX-ACCESS-001 designer fixture inspection captures', async ({ page, context }, info) => {
  const directory = await mkdtemp(join(tmpdir(), 'workbench-design-'));
  try {
    const file = join(directory, 'report.html');
    await offlineFile(file, snapshotFixture());
    await context.setOffline(true);
    await page.goto(pathToFileURL(file).href);
    for (const name of ['Overview', 'Tasks', 'Coverage', 'Actions & jobs']) {
      await page
        .getByRole('navigation', { name: 'Workbench destinations' })
        .getByRole('link', { name, exact: true })
        .click();
      await page.screenshot({
        path: info.outputPath(`${name.replaceAll(/[^a-z]/gi, '-').toLowerCase()}.png`),
        fullPage: true,
      });
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
