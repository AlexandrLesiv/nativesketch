import { readBoard } from './lib/board.ts';
import { root, main } from './lib/context.ts';
await main(async () => {
  const args = process.argv.slice(2);
  if (args.length > 1 || (args.length && args[0] !== '--check'))
    throw new Error('Usage: node scripts/board.ts [--check]');
  const tasks = await readBoard(root);
  if (args.length) console.log(`PASS: task board (${tasks.length} tasks)`);
  else
    for (const task of tasks)
      console.log(
        `${task.id}  ${task.status.padEnd(7)}  ${task.title}  [depends: ${task.depends_on.join(',') || 'none'}]`,
      );
});
