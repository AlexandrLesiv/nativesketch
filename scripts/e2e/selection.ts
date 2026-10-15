/** Filters execution only: discovery must remain the complete native enumeration. */
export function selectedTests(discovered: string[], requested: string[]) {
  if (new Set(requested).size !== requested.length)
    throw new Error('Duplicate native test selection');
  for (const id of requested) {
    if (!/^xctest:NativeSketchUITests:[A-Za-z][A-Za-z0-9_]*\/test[A-Za-z0-9_]+$/.test(id))
      throw new Error(`Invalid native test ID: ${id}`);
    if (!discovered.includes(id)) throw new Error(`Native test was not discovered: ${id}`);
  }
  return requested.length ? [...requested] : [...discovered];
}

export function selectionArguments(selected: string[]) {
  return selected.map((id) => `-only-testing:${id.replace('xctest:', '').replace(':', '/')}`);
}

export function nativeInputRoute(id: string) {
  return id.includes(':NativeGestureCancellationTests/')
    ? 'XCUIAutomation setup/save; canvas-verified session-tap CGEvent held pointer and AX window control; no physical pen'
    : 'XCUIAutomation mouse/keyboard, no physical pen';
}
