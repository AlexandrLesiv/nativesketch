import path from 'node:path';
/** @param {string} root @param {string} target @param {typeof path} [api] */
export function isContained(root, target, api = path) {
  const relative = api.relative(root, target);
  return !api.isAbsolute(relative) && relative !== '..' && !relative.startsWith(`..${api.sep}`);
}
