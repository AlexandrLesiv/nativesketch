import { isContained } from './paths.ts';
import { readFile, readdir, realpath, stat } from 'node:fs/promises';
import { resolve } from 'node:path';
export type Task = {
  id: string;
  title: string;
  status: string;
  brief: string;
  depends_on: string[];
  roles: string[];
};

export async function readBoard(root: string): Promise<Task[]> {
  const board = JSON.parse(await readFile(resolve(root, 'tasks/board.json'), 'utf8'));
  if (board.schema_version !== 1 || !Array.isArray(board.tasks))
    throw new Error('Expected schema_version 1 and tasks list');
  const roles = new Set(
    (await readdir(resolve(root, '.codex/agents')))
      .filter((p) => p.endsWith('.toml'))
      .map((p) => p.slice(0, -5)),
  );
  const statuses = new Set(['ready', 'waiting', 'active', 'review', 'done', 'blocked']);

  const byId: Map<string, Task> = new Map();
  for (const task of board.tasks) {
    if (typeof task.id !== 'string' || !/^\d{3}$/.test(task.id))
      throw new Error('Task IDs must be three-digit strings');
    if (byId.has(task.id)) throw new Error(`Duplicate task ${task.id}`);
    byId.set(task.id, task);
    if (!statuses.has(task.status)) throw new Error(`Invalid status for ${task.id}`);
    if (typeof task.title !== 'string' || !task.title.trim())
      throw new Error(`Missing title for ${task.id}`);
    if (typeof task.brief !== 'string') throw new Error(`Missing brief for ${task.id}`);
    const brief = await realpath(resolve(root, task.brief));
    if (!isContained(await realpath(root), brief) || !(await stat(brief)).isFile())
      throw new Error(`Missing or external brief for ${task.id}`);
    for (const key of ['depends_on', 'roles']) {
      const values = task[key];
      if (
        !Array.isArray(values) ||
        values.some((value) => typeof value !== 'string') ||
        new Set(values).size !== values.length
      )
        throw new Error(`Invalid ${key} for ${task.id}`);
    }
    if (!task.roles.length || task.roles.some((role: string) => !roles.has(role)))
      throw new Error(`Unknown roles for ${task.id}`);
  }
  const visited = new Set();
  const visiting = new Set();

  function visit(id: string) {
    const task = byId.get(id);
    if (!task) throw new Error(`Unknown dependency ${id}`);
    if (visiting.has(id)) throw new Error(`Dependency cycle at ${id}`);
    if (visited.has(id)) return;
    visiting.add(id);
    task.depends_on.forEach(visit);
    visiting.delete(id);
    visited.add(id);
  }
  for (const task of byId.values()) {
    visit(task.id);
    const pending = task.depends_on.filter((id) => byId.get(id)?.status !== 'done');
    if (pending.length && ['ready', 'active', 'review', 'done'].includes(task.status))
      throw new Error(`Task ${task.id} has unfinished dependencies: ${pending.join(',')}`);
  }
  return [...byId.values()];
}
