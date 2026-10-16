import { join } from 'node:path';
import { command, root, requireHost } from '../lib/context.ts';

/** Include package manifests and test sources, excluding ignored build products. */
export async function checkSwift() {
  requireHost('darwin');
  const [listed, removed] = await Promise.all([
    command(
      'git',
      ['ls-files', '-z', '--cached', '--others', '--exclude-standard', '--', '*.swift'],
      {
        capture: true,
      },
    ),
    command('git', ['ls-files', '-z', '--deleted', '--', '*.swift'], { capture: true }),
  ]);
  const deleted = new Set(String(removed).split('\0'));
  const paths = [...new Set(String(listed).split('\0'))]
    .filter((path) => path && !deleted.has(path))
    .map((path) => join(root, path));
  if (!paths.length) throw new Error('No handwritten Swift sources found');
  await command('xcrun', ['swift-format', 'lint', '--strict', ...paths], {
    env: {
      DEVELOPER_DIR: process.env.DEVELOPER_DIR ?? '/Applications/Xcode.app/Contents/Developer',
    },
  });
  console.log(`PASS: strict Swift lint (${paths.length} source, manifest and test files)`);
}
