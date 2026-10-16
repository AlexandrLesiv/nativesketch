import type { Document, Section } from '../contracts/index.ts';
import { boundedRead, hash, safeText } from '../files.ts';
export async function documents(root: string, paths: string[]): Promise<Section<Document[]>> {
  const result: Section<Document[]> = { state: 'ready', diagnostics: [], data: [] };
  const approved = paths.filter(
    (path) =>
      /^(docs\/|tasks\/).*\.(md|txt)$/.test(path) ||
      /^(README|AGENTS)\.md$/.test(path) ||
      /^apps\/[^/]+\/README\.md$/.test(path),
  );
  let bytes = 0;
  if (approved.length > 1000) {
    result.state = 'partial';
    result.diagnostics.push({
      severity: 'error',
      code: 'documents-count',
      message: 'Document count exceeds 1000.',
    });
  }
  for (const id of approved.slice(0, 1000)) {
    try {
      const buffer = await boundedRead(root, id);
      bytes += buffer.length;
      if (bytes > 12 * 1024 * 1024) throw new Error('Document total exceeds 12 MiB');
      const text = safeText(buffer.toString('utf8'));
      result.data.push({
        id,
        title: text.match(/^#\s+(.+)$/m)?.[1] ?? id,
        format: id.endsWith('.md') ? 'markdown' : 'text',
        text,
        sha256: hash(buffer),
      });
    } catch {
      result.state = 'partial';
      result.diagnostics.push({
        severity: 'error',
        code: 'document-unavailable',
        path: id,
        message: 'Document rejected: unreadable, escaping or beyond the configured byte budget.',
      });
    }
    if (bytes > 12 * 1024 * 1024) break;
  }
  return result;
}
