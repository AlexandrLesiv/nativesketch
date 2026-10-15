import { main } from './lib/context.mjs';
import { options } from './lib/options.mjs';
import { build } from './lib/build.mjs';
await main(async () => build(options(process.argv.slice(2))));
