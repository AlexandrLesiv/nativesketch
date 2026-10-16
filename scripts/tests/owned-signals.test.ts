import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { signalGroups, confirmGroupsExited } from '../lib/owned-signals.ts';
import {
  allocateOwnership,
  register,
  releaseOwnership,
  ownershipVariable,
} from '../lib/owned-registry.ts';
function failure(code: string) {
  return Object.assign(new Error(code), { code });
}
await test('owned group permission failure does not prevent signalling other registered groups', (context) => {
  const calls: number[] = [];
  context.mock.method(process, 'kill', (pid: number) => {
    calls.push(pid);
    if (pid === -123) throw failure('EPERM');
    return true;
  });
  const diagnostics = signalGroups([{ pid: 123 }, { pid: 456 }], 'SIGKILL');
  assert.deepEqual(calls, [-123, -456]);
  assert.equal(diagnostics.length, 1);
  assert.equal(diagnostics[0].code, 'EPERM');
});
await test('owned transient permission diagnostics require every group to reach ESRCH', (context) => {
  const calls = new Map<number, number>();
  context.mock.method(process, 'kill', (pid: number) => {
    const count = (calls.get(pid) ?? 0) + 1;
    calls.set(pid, count);
    if (count > 1) throw failure('ESRCH');
    if (pid === -123) throw failure('EPERM');
    return true;
  });
  const diagnostics = confirmGroupsExited([{ pid: 123 }, { pid: 456 }]);
  assert.equal(diagnostics.length, 1);
  assert.equal(diagnostics[0].phase, 'exit-probe');
  assert.equal(calls.get(-123), 2);
  assert.equal(calls.get(-456), 2);
});
await test(
  'owned persistent permission failure cannot release ownership registrations',
  { skip: process.platform === 'win32' },
  (context) => {
    const previous = process.env[ownershipVariable];
    delete process.env[ownershipVariable];
    const { owner } = allocateOwnership();
    register(owner, null, process.pid);
    context.mock.method(process, 'kill', () => {
      throw failure('EPERM');
    });
    try {
      assert.throws(() => releaseOwnership(owner, true), /did not finish.*EPERM/);
      assert.equal(existsSync(join(owner.directory, `${owner.scope}.json`)), true);
    } finally {
      if (previous === undefined) delete process.env[ownershipVariable];
      else process.env[ownershipVariable] = previous;
      rmSync(owner.directory, { recursive: true });
    }
  },
);
await test('owned live groups and non-permission probe errors remain fatal', (context) => {
  const mocked = context.mock.method(process, 'kill', () => true);
  assert.throws(() => confirmGroupsExited([{ pid: 123 }]), /did not finish/);
  mocked.mock.mockImplementation(() => {
    throw failure('EINVAL');
  });
  assert.throws(() => confirmGroupsExited([{ pid: 123 }]), /EINVAL/);
});
