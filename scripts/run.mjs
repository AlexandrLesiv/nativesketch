import { join } from 'node:path';
import { main, command, root, controller } from './lib/context.mjs';
import { options } from './lib/options.mjs';
import { buildMacos } from './platforms/macos.mjs';
import { buildWeb } from './platforms/web.mjs';
import { runApple } from './platforms/apple.mjs';
import { serve } from './lib/server.mjs';
await main(async () => {
  const args = options(process.argv.slice(2));
  if (args.platform === 'macos') await command('open', [await buildMacos(args.profile)]);
  else if (args.platform === 'web') {
    await buildWeb();
    const port = Number(process.env.PORT ?? 8077);
    if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error('Invalid PORT');
    await serve(join(root, 'artifacts/web'), port, controller.signal);
  } else await runApple(args.platform, args.device, args.args);
});
