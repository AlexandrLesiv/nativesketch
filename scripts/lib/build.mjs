import { buildMacos } from '../platforms/macos.mjs';
import { buildApple } from '../platforms/apple.mjs';
import { buildAndroid } from '../platforms/android.mjs';
import { buildLinux, buildWindows } from '../platforms/desktop.mjs';
import { buildWeb, buildWasm } from '../platforms/web.mjs';
/** @param {ReturnType<typeof import('./options.mjs').options>} options */
export async function build(options) {
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
