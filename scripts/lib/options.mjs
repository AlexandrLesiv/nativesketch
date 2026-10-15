/** @param {string[]} args */
export function options(args) {
  const result = {
    platform: '',
    profile: 'debug',
    variant: 'simulator',
    device: '',
    core: false,
    args: /** @type {string[]} */ ([]),
  };
  while (args.length) {
    const flag = args.shift();
    if (flag === '--') {
      result.args = args;
      break;
    }
    if (flag === '--core') {
      result.core = true;
      continue;
    }
    if (!['--platform', '--profile', '--variant', '--device'].includes(flag ?? ''))
      throw new Error(`Unknown option ${flag}`);
    const value = args.shift();
    if (!value || value.startsWith('--')) throw new Error(`Missing value for ${flag}`);
    const key = /** @type {'platform'|'profile'|'variant'|'device'} */ (flag?.slice(2));
    result[key] = value;
  }
  return result;
}
