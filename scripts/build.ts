import { main } from './lib/context.ts';
import { options } from './lib/options.ts';
import { build } from './lib/build.ts';
await main(async () => build(options(process.argv.slice(2))));
