import { main } from './lib/context.ts';
import { checkLegacyFfi } from './platforms/legacy-ffi.ts';

await main(async () => {
  const args = process.argv.slice(2);
  if (args.length && (args.length !== 2 || args[0] !== '--output' || !args[1]))
    throw new Error('Usage: node scripts/test-legacy-ffi.ts [--output NEW_DIRECTORY]');
  await checkLegacyFfi(args[1]);
});
