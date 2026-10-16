import { copyFile, mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

export async function seedCorpus(root: string, output: string, target: string) {
  const directory = join(output, `${target}-corpus`);
  await mkdir(directory);
  if (target === 'document_codec') {
    await copyFile(
      join(root, 'crates/sketch-core/tests/fixtures/portable-v1.nsketch'),
      join(directory, 'portable-v1'),
    );
    await writeFile(join(directory, 'truncated'), Buffer.from('NSKETCH1'));
  } else {
    const tags = [0, 1, 1, 2, 4, 5, 9, 10, 11, 7, 8, 6, 7, 3, 12, 13, 14, 15];
    for (const [name, flags] of [
      ['current', 0],
      ['stale', 1],
      ['invalid-batch', 32],
    ] as const) {
      const bytes = tags.flatMap((tag) => [tag, flags, 141, 140, 121, 142, 0, 0]);
      await writeFile(join(directory, name), Buffer.from(bytes));
    }
  }
  return directory;
}
