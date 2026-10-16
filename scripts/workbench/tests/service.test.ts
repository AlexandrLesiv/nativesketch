import test from 'node:test';
import assert from 'node:assert/strict';
import { fixture } from './fixture.ts';
import { startWorkbench } from '../service.ts';
import { Jobs } from '../jobs.ts';
import { setTimeout } from 'node:timers/promises';
await test('workbench service rejects foreign origins capabilities foreground and second owners', async () => {
  const f = await fixture();
  const service = await startWorkbench({ root: f.root, assets: f.assets });
  try {
    await assert.rejects(startWorkbench({ root: f.root, assets: f.assets }), /EEXIST/);
    assert.equal((await fetch(service.url + 'api/session')).status, 403);
    assert.equal(
      (await fetch(service.url + 'api/session', { headers: { 'Sec-Fetch-Site': 'cross-site' } }))
        .status,
      403,
    );
    const session = await (
      await fetch(service.url + 'api/session', { headers: { 'Sec-Fetch-Site': 'same-origin' } })
    ).json();
    assert.equal((await fetch(service.url + 'api/snapshot')).status, 403);
    const headers = {
      'X-Workbench-Capability': session.capability,
      Origin: service.url.slice(0, -1),
      'Content-Type': 'application/json',
    };
    assert.equal((await fetch(service.url + 'api/snapshot', { headers })).status, 200);
    for (const actionId of ['native-tests', 'run-macos', 'arbitrary-shell'])
      assert.equal(
        (
          await fetch(service.url + 'api/jobs', {
            method: 'POST',
            headers,
            body: JSON.stringify({ actionId }),
          })
        ).status,
        403,
      );
    assert.equal(
      (
        await fetch(service.url + 'api/jobs', {
          method: 'POST',
          headers: { ...headers, Origin: 'http://evil.invalid' },
          body: '{"actionId":"check-core"}',
        })
      ).status,
      403,
    );
    assert.equal(
      (
        await fetch(service.url + 'api/jobs', {
          method: 'POST',
          headers,
          body: '{"actionId":"check-core","argv":[]}',
        })
      ).status,
      403,
    );
    assert.equal(
      (
        await fetch(service.url + 'api/jobs', {
          method: 'POST',
          headers,
          body: JSON.stringify({ actionId: 'x'.repeat(5000) }),
        })
      ).status,
      403,
    );
    assert.equal(
      (await fetch(service.url + 'api/artifacts/' + encodeURIComponent('../../.env'), { headers }))
        .status,
      403,
    );
  } finally {
    await service.close();
    await f.close();
  }
});
await test('workbench queue limits simultaneous admissions and cancellation before spawn', async () => {
  const f = await fixture();
  const jobs = new Jobs(f.root);
  try {
    const attempted = await Promise.allSettled(
      Array.from({ length: 12 }, () => jobs.enqueue('check-core')),
    );
    assert.ok(attempted.filter((row) => row.status === 'fulfilled').length <= 5);
    assert.ok(jobs.list().filter((job) => job.state === 'queued').length <= 4);
    for (const job of jobs.list()) jobs.cancel(job.id);
    await jobs.close();
    assert.ok(jobs.list().every((job) => !['queued', 'running', 'cancelling'].includes(job.state)));
    assert.ok(jobs.list().some((job) => job.state === 'cancelled'));
  } finally {
    await jobs.close();
    await f.close();
  }
});
await test('workbench real failed job stays failed with retained safe output', async () => {
  const f = await fixture();
  const jobs = new Jobs(f.root);
  try {
    const queued = await jobs.enqueue('check-core');
    for (let i = 0; i < 100 && !jobs.list().find((job) => job.id === queued.id)?.endedAt; i++)
      await setTimeout(20);
    const job = jobs.list().find((item) => item.id === queued.id);
    assert.equal(job?.state, 'failed');
    assert.equal(job?.exitCode, 7);
    assert.match(job?.log ?? '', /disposable failure/);
    assert.equal(job?.artifactIds.length, 1);
  } finally {
    await jobs.close();
    await f.close();
  }
});
