import { test, expect } from '@playwright/test';
import { ready, stroke, download } from './helpers.ts';
test('WEB-ACTION-001 unavailable Undo retains draft and native field editing retains focus', async ({
  page,
}) => {
  await ready(page);
  const box = (await page.locator('#canvas').boundingBox())!;
  await page.mouse.move(box.x + 100, box.y + 100);
  await page.mouse.down();
  await page.mouse.move(box.x + 150, box.y + 140);
  await page.keyboard.press('Control+z');
  await page.mouse.up();
  await expect(page.locator('#status')).toContainText('1 stroke');
  await page.locator('#width').fill('12');
  await page.keyboard.press('Control+z');
  await expect(page.locator('#width')).toBeFocused();
  await expect(page.locator('#status')).toContainText('1 stroke');
});
test('WEB-EXPORT-001 guard Download stays and clean reopened download stays clean', async ({
  page,
}, info) => {
  await ready(page);
  await stroke(page);
  await page.locator('#new').click();
  await page.screenshot({ path: info.outputPath('unsaved-guard.png') });
  await info.attach('unsaved-guard', {
    path: info.outputPath('unsaved-guard.png'),
    contentType: 'image/png',
  });
  const pending = page.waitForEvent('download');
  await page.locator('#guard').getByRole('button', { name: 'Download copy' }).click();
  const saved = await pending,
    path = info.outputPath('guard-copy.nsketch');
  await saved.saveAs(path);
  await expect(page.locator('#status')).toContainText('1 stroke');
  await expect(page.locator('#status')).toContainText('Unsaved');
  await page.locator('#open').click();
  const chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'Discard and continue' }).click();
  await (await chooser).setFiles(path);
  await expect(page.locator('#status')).not.toContainText('Unsaved');
  await download(page, info, 'clean-copy.nsketch');
  await expect(page.locator('#status')).not.toContainText('Unsaved');
});
test('WEB-PICKER-001 real chooser requests from Open and shortcut survive injected cancellation', async ({
  page,
}) => {
  await ready(page);
  for (const shortcut of [false, true, false]) {
    const chooser = page.waitForEvent('filechooser');
    if (shortcut) {
      await page.locator('#canvas').focus();
      await page.keyboard.press('Control+o');
    } else await page.locator('#open').click();
    await (await chooser).setFiles([]);
    await page.locator('#file').dispatchEvent('cancel');
    await expect(page.locator('#canvas')).toBeFocused();
    await expect(page.locator('#status')).toContainText('0 strokes');
  }
});
test('WEB-STALE-001 delayed File read cannot replace newer drawing or steal native field focus', async ({
  page,
}, info) => {
  await ready(page);
  await stroke(page);
  const saved = await download(page, info, 'stale-copy.nsketch');
  await page.evaluate(() => {
    File.prototype.arrayBuffer = async function () {
      document.body.dataset.readStarted = 'true';
      await new Promise<void>((resolve) =>
        window.addEventListener('release-test-read', () => resolve(), { once: true }),
      );
      const bytes = await Blob.prototype.arrayBuffer.call(this);
      setTimeout(() => {
        document.body.dataset.readCompleted = 'true';
      }, 0);
      return bytes;
    };
  });
  await page.locator('#open').click();
  const chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'Discard and continue' }).click();
  await (await chooser).setFiles(saved.path);
  await expect(page.locator('body')).toHaveAttribute('data-read-started', 'true');
  await stroke(page, 70);
  await page.locator('#width').focus();
  await page.evaluate(() => window.dispatchEvent(new Event('release-test-read')));
  await expect(page.locator('#status')).toContainText('2 strokes');
  await expect(page.locator('body')).toHaveAttribute('data-read-completed', 'true');
  await expect(page.locator('#width')).toBeFocused();
  await expect(page.locator('#status')).toContainText('2 strokes');
  await expect(page.locator('#error')).not.toBeVisible();
});
