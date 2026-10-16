import { root } from './lib/context.ts';
import { browserOptions } from './browser-e2e/options.ts';
import { workbenchJourney } from './workbench-e2e/journey.ts';
await workbenchJourney(browserOptions(process.argv.slice(2), root));
