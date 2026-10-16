import { main } from './lib/context.ts';
import { lintTypeScript } from './quality/typescript.ts';
await main(lintTypeScript);
