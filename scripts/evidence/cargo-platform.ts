/** Cargo packages containing native effects opt into the actual execution host.
 * This declaration cannot claim a different OS or turn missing tests into passes. */
export function cargoTestPlatform(declaration: unknown, host: string): string {
  if (declaration === undefined || declaration === 'portable') return 'portable';
  if (declaration !== 'host') throw new Error('Unknown Cargo test-platform declaration');
  switch (host) {
    case 'darwin':
      return 'macos';
    case 'win32':
      return 'windows';
    case 'linux':
      return 'linux';
    default:
      throw new Error(`Unsupported native Cargo evidence host: ${host}`);
  }
}
