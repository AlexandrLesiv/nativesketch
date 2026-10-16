import {
  mkdtempSync,
  readFileSync,
  writeFileSync,
  readdirSync,
  lstatSync,
  unlinkSync,
  rmSync,
  openSync,
  closeSync,
  renameSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join, dirname, basename } from 'node:path';
import { randomUUID, randomBytes } from 'node:crypto';
export const ownershipVariable = 'NATIVE_SKETCH_INTERNAL_PROCESS_OWNER';
export interface Ownership {
  directory: string;
  token: string;
  scope: string;
}
interface Entry {
  scope: string;
  parent: string | null;
  pid: number;
}
const uuid = /^[a-f0-9-]{36}$/;
export function ownership(value: string): Ownership {
  const item: Ownership = JSON.parse(value);
  if (
    !item ||
    typeof item.directory !== 'string' ||
    !/^[a-f0-9]{48}$/.test(item.token) ||
    !uuid.test(item.scope)
  )
    throw new Error('Invalid internal process ownership');
  if (
    dirname(item.directory) !== tmpdir() ||
    !basename(item.directory).startsWith('native-sketch-owned-')
  )
    throw new Error('Invalid ownership directory');
  const info = lstatSync(item.directory);
  if (
    !info.isDirectory() ||
    info.isSymbolicLink() ||
    (info.mode & 0o077) !== 0 ||
    info.uid !== process.getuid?.()
  )
    throw new Error('Unsafe ownership directory');
  if (readFileSync(join(item.directory, 'token'), 'utf8') !== item.token)
    throw new Error('Invalid ownership token');
  return item;
}
function entries(owner: Ownership): Entry[] {
  const names = readdirSync(owner.directory);
  if (names.length > 1025) throw new Error('Process ownership entry bound');
  return names
    .filter((name) => name.endsWith('.json'))
    .map((name) => {
      try {
        const path = join(owner.directory, name);
        const info = lstatSync(path);
        if (!info.isFile() || info.isSymbolicLink() || info.size > 512)
          throw new Error('Invalid ownership entry');
        const item: Entry = JSON.parse(readFileSync(path, 'utf8'));
        if (
          name !== `${item.scope}.json` ||
          !uuid.test(item.scope) ||
          (item.parent !== null && !uuid.test(item.parent)) ||
          !Number.isSafeInteger(item.pid) ||
          item.pid < 2
        )
          throw new Error('Invalid ownership entry');
        return item;
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code === 'ENOENT') return null;
        throw error;
      }
    })
    .filter((item): item is Entry => item !== null);
}
export function assertMayStart(owner: Ownership) {
  const all = entries(owner);
  let scope: string | null = owner.scope;
  for (let depth = 0; scope !== null; depth++) {
    if (depth >= 32) throw new Error('Process ownership depth bound');
    const entry = all.find((item) => item.scope === scope);
    if (!entry) throw new Error('Process ownership parent missing');
    try {
      lstatSync(join(owner.directory, `${scope}.cancel`));
      throw new Error('Process ownership is cancelling');
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
    }
    scope = entry.parent;
  }
}
export function allocateOwnership(): { owner: Ownership; parent: string | null; root: boolean } {
  const inherited = process.env[ownershipVariable];
  if (inherited) {
    const owner = ownership(inherited);
    assertMayStart(owner);
    if (entries(owner).length >= 256) throw new Error('Active process group bound');
    return { owner: { ...owner, scope: randomUUID() }, parent: owner.scope, root: false };
  }
  const directory = mkdtempSync(join(tmpdir(), 'native-sketch-owned-'));
  const token = randomBytes(24).toString('hex');
  writeFileSync(join(directory, 'token'), token, { mode: 0o600, flag: 'wx' });
  return { owner: { directory, token, scope: randomUUID() }, parent: null, root: true };
}
export function register(owner: Ownership, parent: string | null, pid: number) {
  const lock = join(owner.directory, 'admission.lock');
  let descriptor: number | undefined;
  for (let attempt = 0; attempt < 250 && descriptor === undefined; attempt++) {
    try {
      descriptor = openSync(lock, 'wx', 0o600);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'EEXIST') throw error;
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 2);
    }
  }
  if (descriptor === undefined) throw new Error('Process admission lock unavailable');
  const temporary = join(owner.directory, `.pending-${owner.scope}`);
  try {
    if (parent) assertMayStart({ ...owner, scope: parent });
    if (entries(owner).length >= 256) throw new Error('Active process group bound');
    writeFileSync(temporary, JSON.stringify({ scope: owner.scope, parent, pid }), {
      mode: 0o600,
      flag: 'wx',
    });
    renameSync(temporary, join(owner.directory, `${owner.scope}.json`));
  } finally {
    closeSync(descriptor);
    unlinkSync(lock);
    rmSync(temporary, { force: true });
  }
}

function subtree(owner: Ownership) {
  const all = entries(owner);
  const selected = new Set([owner.scope]);
  for (let depth = 0; depth < 32; depth++) {
    const before = selected.size;
    for (const item of all) if (item.parent && selected.has(item.parent)) selected.add(item.scope);
    if (before === selected.size) return all.filter((item) => selected.has(item.scope));
  }
  throw new Error('Process ownership depth bound');
}
export function signalOwned(owner: Ownership, signal: NodeJS.Signals) {
  writeFileSync(join(owner.directory, `${owner.scope}.cancel`), '', { mode: 0o600 });
  const selected = subtree(owner);
  for (const item of selected.reverse()) {
    try {
      process.kill(-item.pid, signal);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ESRCH') throw error;
    }
  }
}
export function releaseOwnership(owner: Ownership, root: boolean) {
  // Never adopt persisted registrations after owner death or remove a still-live group.
  const selected = subtree(owner);
  let remaining = selected;
  for (let attempt = 0; attempt < 200 && remaining.length; attempt++) {
    remaining = remaining.filter((item) => {
      try {
        process.kill(-item.pid, 0);
        return true;
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code === 'ESRCH') return false;
        throw error;
      }
    });
    if (remaining.length) Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 5);
  }
  if (remaining.length) throw new Error('Owned groups did not finish within cleanup bound');
  for (const item of selected)
    for (const suffix of ['json', 'cancel']) {
      try {
        unlinkSync(join(owner.directory, `${item.scope}.${suffix}`));
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
      }
    }
  if (root) rmSync(owner.directory, { recursive: true });
}
