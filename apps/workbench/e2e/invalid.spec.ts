import { test, expect } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { snapshotFixture } from './fixture.ts';
import { offlineFile } from './offline.ts';
test('DX-TASK-001 cyclic invalid tasks retain readable table and do not fabricate order', async ({
  page,
  context,
}) => {
  const directory = await mkdtemp(join(tmpdir(), 'workbench-cycle-'));
  try {
    const snapshot = snapshotFixture();
    snapshot.tasks.state = 'invalid';
    snapshot.tasks.diagnostics.push({ severity: 'error', code: 'cycle', message: 'Fixture cycle' });
    const first = snapshot.tasks.data[0];
    if (!first) throw new Error('Missing fixture task');
    first.depends_on = ['002'];
    const file = join(directory, 'cycle.html');
    await offlineFile(file, snapshot);
    await context.setOffline(true);
    await page.goto(pathToFileURL(file).href);
    await page.getByRole('link', { name: 'Tasks', exact: true }).click();
    await page.getByText('Show dependency graph', { exact: true }).click();
    await expect(page.getByRole('main')).toContainText('Dependency cycle');
    await expect(page.getByRole('main')).toContainText('Section state: invalid');
    await expect(page.getByRole('group', { name: 'Task dependencies' })).toHaveCount(0);
    await expect(page.getByRole('table')).toContainText('Core commands');
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
