/** Resolve only exact aliases; class names can contain module qualification dots. */
export function testIdentifier(value: string, discovered: string[]): string | undefined {
  const matches = discovered.filter((id) => {
    const [prefix, target, path] = id.split(':');
    if (prefix !== 'xctest' || !target || !path) throw new Error('Malformed XCTest identity');
    const slash = path.lastIndexOf('/');
    const dotted = `${path.slice(0, slash)}.${path.slice(slash + 1)}`;
    return [
      path,
      `${path}()`,
      dotted,
      `${dotted}()`,
      `${target}/${path}`,
      `${target}/${path}()`,
    ].includes(value);
  });
  if (matches.length > 1) throw new Error('Ambiguous XCTest identity');
  return matches[0];
}
