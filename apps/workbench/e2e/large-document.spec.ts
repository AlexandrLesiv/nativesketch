import { test, expect } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { snapshotFixture } from './fixture.ts';
import { offlineFile } from './offline.ts';
test('DX-DOC-001 oversized Markdown rendering retains full text in bounded DOM', async ({
  page,
}) => {
  const directory = await mkdtemp(join(tmpdir(), 'workbench-large-doc-'));
  try {
    const snapshot = snapshotFixture();
    const doc = snapshot.documents.data[0];
    if (!doc) throw new Error('Missing fixture');
    doc.text = Array.from({ length: 6000 }, (_, i) => `# Section ${i}`).join('\n');
    const file = join(directory, 'large.html');
    await offlineFile(file, snapshot);
    await page.goto(pathToFileURL(file).href + '#view=docs&selected=docs%2Farchitecture.md');
    await expect(page.locator('article')).toContainText(
      'Large document: rendered as complete plain text',
    );
    await expect(page.locator('article pre')).toContainText('Section 5999');
    await expect(page.locator('article *')).toHaveCount(2);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
