import { root } from './lib/context.ts';
import { browserOptions } from './browser-e2e/options.ts';
import { browserJourney } from './browser-e2e/journey.ts';
await browserJourney(browserOptions(process.argv.slice(2), root));
