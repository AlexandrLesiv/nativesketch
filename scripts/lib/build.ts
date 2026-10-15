import type { options as parseOptions } from './options.ts';
import { buildMacos } from '../platforms/macos.ts';
import { buildApple } from '../platforms/apple.ts';
import { buildAndroid } from '../platforms/android.ts';
import { buildLinux, buildWindows } from '../platforms/desktop.ts';
import { buildWeb, buildWasm } from '../platforms/web.ts';

export async function build(options: ReturnType<typeof parseOptions>) {
  switch (options.platform) {
    case 'macos':
      return buildMacos(options.profile);
    case 'ios':
    case 'watchos':
    case 'tvos':
    case 'visionos':
    case 'apple-family':
      return buildApple(options.platform, options.variant);
    case 'android':
      return buildAndroid();
    case 'linux':
      return buildLinux();
    case 'windows':
      return buildWindows();
    case 'web':
      return buildWeb();
    case 'wasm':
      return buildWasm();
    default:
      throw new Error(
        'Specify --platform macos|ios|watchos|tvos|visionos|apple-family|android|linux|windows|web|wasm',
      );
  }
}
