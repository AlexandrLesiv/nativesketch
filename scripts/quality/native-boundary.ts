import { posix } from 'node:path';

const app = 'apps/macos/Sources/NativeSketch/';
const modulePath = 'apps/macos/Sources/CSketchSession/module.modulemap';
const headers = 'crates/sketch-native-ffi/include/';
const manifest = 'apps/macos/Package.swift';

export function isNativeBoundaryPath(path: string) {
  return (
    path === manifest ||
    path === modulePath ||
    (path.startsWith(app) && path.endsWith('.swift')) ||
    (path.startsWith(headers) && path.endsWith('.h'))
  );
}

/** Regression guard for the supported native module boundary, not a Swift/C parser
 * or proof of semantic routing. Actual compiler consumers and behavior tests remain required. */
export function checkNativeBoundary(files: ReadonlyMap<string, string>) {
  const failures: string[] = [];
  const fail = (path: string, reason: string) => failures.push(`${path}: ${reason}`);
  const packageText = files.get(manifest) ?? '';
  if (!packageText.includes('"CSketchSession"') || !packageText.includes('libsketch_native_ffi.a'))
    fail(manifest, 'native app must declare the versioned session module and library');
  if (/CSketchFFI|\b(?:lib)?sketch_ffi\b/.test(packageText))
    fail(manifest, 'legacy native bridge dependency is forbidden');
  for (const [path, text] of files) {
    if (!path.startsWith(app) || !path.endsWith('.swift')) continue;
    if (
      /\bimport\s+(?:(?:class|enum|func|struct|var|typealias|protocol)\s+)?CSketchFFI\b/.test(text)
    )
      fail(path, 'production Swift must not import CSketchFFI');
    if (/@_silgen_name\b/.test(text))
      fail(path, 'manual C symbol declarations bypass the typed native module');
  }
  const moduleText = files.get(modulePath) ?? '';
  const roots = [...moduleText.matchAll(/\bheader\s+"([^"]+)"/g)].map((match) =>
    posix.normalize(posix.join(posix.dirname(modulePath), match[1])),
  );
  if (
    !/\bmodule\s+CSketchSession\s*\[system\]/.test(moduleText) ||
    (moduleText.match(/\bmodule\b/g)?.length ?? 0) !== 1 ||
    /\b(?:umbrella|use|extern|link)\b/.test(moduleText) ||
    roots.length !== 1 ||
    roots[0] !== `${headers}sketch_session_v1.h`
  )
    fail(modulePath, 'production module must expose only the versioned session umbrella');
  const seen = new Set<string>();
  const visit = (path: string) => {
    if (seen.has(path)) return;
    seen.add(path);
    if (!path.startsWith(headers) || path === `${headers}sketch_test_v1.h`) {
      fail(path, 'production include graph reaches legacy, external or test-only headers');
      return;
    }
    const text = files.get(path);
    if (text === undefined) {
      fail(path, 'missing production bridge header');
      return;
    }
    for (const directive of text.matchAll(/^\s*#\s*(?:include|import)\b([^\n]*)/gm)) {
      const include = directive[1].trim();
      const quoted = /^"([^"]+)"\s*(?:\/\/.*)?$/.exec(include);
      if (quoted) visit(posix.normalize(posix.join(posix.dirname(path), quoted[1])));
      else if (include !== '<stdint.h>')
        fail(path, 'unapproved or indirect production bridge include');
    }
  };
  for (const root of roots) visit(root);
  return failures;
}
