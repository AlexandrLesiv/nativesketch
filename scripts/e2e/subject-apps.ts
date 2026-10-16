import type { Source } from '../evidence/types.ts';
import { cp, realpath, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { command, root } from '../lib/context.ts';
import { fileHash } from '../evidence/identity.ts';

export type AppIdentity = { path: string; bundleIdentifier: string; binarySha256: string };

/** Isolated disposable apps: the helper is a separate process using the same tested build. */
export async function prepareSubjectApps(
  output: string,
  source: Source,
  builtApp = join(root, 'artifacts/NativeSketch.app'),
  execute = command,
) {
  const isolation = createHash('sha256')
    .update(root + JSON.stringify(source))
    .digest('hex')
    .slice(0, 24);
  const bundleIdentifier = `dev.nativesketch.e2e.${isolation}`;
  async function copy(sourcePath: string, name: string, identifier: string): Promise<AppIdentity> {
    const destination = join(output, name);
    await cp(sourcePath, destination, {
      recursive: true,
      force: false,
      errorOnExist: true,
      verbatimSymlinks: true,
    });
    const path = await realpath(destination);
    const executable = 'Contents/MacOS/NativeSketch';
    if ((await fileHash(join(path, executable))) !== (await fileHash(join(sourcePath, executable))))
      throw new Error('Prepared app executable differs from its source before signing');
    const plist = join(path, 'Contents/Info.plist');
    await execute('plutil', ['-replace', 'CFBundleIdentifier', '-string', identifier, plist]);
    await execute('codesign', ['--force', '--sign', '-', path]);
    await execute('codesign', ['--verify', '--strict', path]);
    const actual = (
      await execute('plutil', ['-extract', 'CFBundleIdentifier', 'raw', '-o', '-', plist], {
        capture: true,
      })
    ).trim();
    if (actual !== identifier) throw new Error(`Prepared app bundle identity mismatch: ${name}`);
    return {
      path,
      bundleIdentifier: actual,
      binarySha256: await fileHash(join(path, 'Contents/MacOS/NativeSketch')),
    };
  }
  const subject = await copy(builtApp, 'Subject.app', bundleIdentifier);
  const focusApp = {
    ...(await copy(subject.path, 'FocusSubject.app', `${bundleIdentifier}.focus`)),
    sourceBinarySha256: subject.binarySha256,
    role: 'disposable-focus-helper',
  };
  await writeFile(join(output, 'focus-app-identity.json'), JSON.stringify(focusApp, null, 2));
  return { subject, focusApp };
}

export function configureSubjectApps(
  environment: Record<string, string>,
  subject: string,
  focus: AppIdentity,
) {
  environment.NATIVE_SKETCH_APP = subject;
  environment.NATIVE_SKETCH_FOCUS_APP = focus.path;
  environment.NATIVE_SKETCH_FOCUS_BINARY_SHA256 = focus.binarySha256;
}
