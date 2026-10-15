import path from 'node:path';

export function isContained(root: string, target: string, api: typeof path = path) {
  const relative = api.relative(root, target);
  return !api.isAbsolute(relative) && relative !== '..' && !relative.startsWith(`..${api.sep}`);
}
