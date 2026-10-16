import { opendir } from 'node:fs/promises';
import { readBoard } from '../../lib/board.ts';
import type { Section, Task } from '../contracts/index.ts';
import { boundedRead, confined } from '../files.ts';
function strings(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === 'string');
}
export async function tasks(root: string): Promise<Section<Task[]>> {
  const result: Section<Task[]> = { state: 'ready', diagnostics: [], data: [] };
  try {
    const text = (await boundedRead(root, 'tasks/board.json')).toString();
    const raw: unknown = JSON.parse(text);
    if (raw && typeof raw === 'object' && 'tasks' in raw && Array.isArray(raw.tasks)) {
      for (const task of raw.tasks) {
        if (
          task &&
          typeof task === 'object' &&
          typeof task.id === 'string' &&
          typeof task.title === 'string' &&
          typeof task.status === 'string' &&
          typeof task.brief === 'string' &&
          strings(task.depends_on) &&
          strings(task.roles)
        )
          result.data.push({
            id: task.id,
            title: task.title,
            status: task.status,
            brief: task.brief,
            depends_on: task.depends_on,
            roles: task.roles,
          });
      }
    }
    const roles: string[] = [];
    let entries = 0;
    for await (const entry of await opendir(await confined(root, '.codex/agents'))) {
      if (++entries > 1000) throw new Error('Role entry bound');
      if (entry.name.endsWith('.toml')) roles.push(entry.name.slice(0, -5));
    }
    result.data = await readBoard(root, { text, roles });
  } catch {
    result.state = 'invalid';
    result.diagnostics.push({
      severity: 'error',
      code: 'board-invalid',
      path: 'tasks/board.json',
      message:
        'Board validation failed, including dependency/role constraints. Readable rows are preserved without inferred ordering or status.',
    });
  }
  return result;
}
