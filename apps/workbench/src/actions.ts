import type {
  Job,
  PreviewResponse,
  ProjectSnapshot,
} from '../../../scripts/workbench/contracts/index.ts';
import { button, detail, el, notice, textList } from './dom.ts';
import { filterForm, matches } from './route.ts';
import { identity, sectionState } from './presentation.ts';
export interface ActionContext {
  live: boolean;
  connected: boolean;
  jobs: Job[];
  preview: PreviewResponse | null;
  start: (id: string) => void;
  cancel: (id: string) => void;
  artifact: (id: string) => void;
}
export function actionsView(snapshot: ProjectSnapshot, context: ActionContext): HTMLElement {
  const root = el('section');
  root.append(
    sectionState(snapshot.actions),
    filterForm('actions'),
    notice(
      'Foreground execution is disabled: app launches, native UI tests, GPU benchmarks and IDE opening cannot run.',
    ),
  );
  if (!context.live)
    root.append(notice('Static report: all execution, refresh and preview controls are disabled.'));
  else if (!context.connected)
    root.append(
      notice('Service disconnected: actions disabled; job outcomes are unknown until reconciled.'),
    );
  for (const action of snapshot.actions.data.filter((item) =>
    matches(`${item.label} ${item.platform} ${item.category}`),
  )) {
    const card = el('section', '', 'card');
    card.dataset.action = action.id;
    const reason = !context.live
      ? 'Static report is read-only'
      : !context.connected
        ? 'Service disconnected'
        : action.foreground
          ? 'Foreground execution disabled by service policy'
          : !action.enabled
            ? (action.disabledReason ?? 'Action unavailable')
            : '';
    const run = button(`Run ${action.label}`, () => context.start(action.id));
    run.disabled = Boolean(reason);
    card.append(
      el('h2', action.label),
      el('p', `${action.platform} · ${action.category}`),
      el('pre', action.commandPreview),
      textList(action.prerequisites),
      run,
    );
    if (reason) card.append(notice(reason));
    root.append(card);
  }
  const jobs = el('section');
  jobs.id = 'jobs-region';
  jobs.append(el('h2', 'Jobs'));
  if (!context.jobs.length)
    jobs.append(
      notice(
        context.live
          ? 'No local jobs recorded.'
          : 'Live job history is not included in static reports.',
      ),
    );
  const active = new Set(['queued', 'running', 'cancelling']);
  const orderedJobs = [...context.jobs].sort(
    (a, b) => Number(active.has(b.state)) - Number(active.has(a.state)),
  );
  for (const job of orderedJobs) {
    const card = el('section', '', 'card');
    card.dataset.job = job.id;
    card.tabIndex = -1;
    card.append(
      el('h3', `${job.actionId} · ${job.state}`),
      identity(job.source),
      el(
        'p',
        `Queued ${job.queuedAt}; started ${job.startedAt ?? 'not started'}; ended ${job.endedAt ?? 'not ended'}`,
      ),
      el('pre', job.commandPreview),
      el('p', `Exit ${job.exitCode ?? 'unavailable'}; signal ${job.signal ?? 'none'}`),
    );
    if (job.sourceChanged)
      card.append(
        notice('Source changed during execution. This job retains its original source identity.'),
      );
    if (job.reason) card.append(notice(job.reason));
    const log = el('pre', job.log);
    log.dataset.log = job.id;
    const output = detail(`Output (${job.outputBytes} bytes)`, log);
    output.dataset.inspection = 'job-output';
    card.append(
      output,
      ...job.artifactIds.map((id) => {
        const download = button('Download artifact ' + id, () => context.artifact(id));
        download.disabled = !context.live || !context.connected;
        return download;
      }),
    );
    if (['queued', 'running', 'cancelling'].includes(job.state)) {
      const cancel = button(`Cancel ${job.id}`, () => context.cancel(job.id));
      cancel.disabled = !context.live || !context.connected || job.state === 'cancelling';
      card.append(cancel);
    }
    jobs.append(card);
  }
  root.insertBefore(jobs, root.children[3] ?? null);
  root.append(previewView(context));
  return root;
}
function previewView(context: ActionContext): HTMLElement {
  const root = el('section');
  root.append(el('h2', 'Drawing app preview'));
  const preview = context.preview?.preview;
  if (!context.live || !preview) {
    root.append(
      notice(
        !context.live
          ? 'Preview execution is unavailable offline.'
          : (context.preview?.reason ?? 'No built preview available.'),
      ),
    );
    return root;
  }
  try {
    const url = new URL(preview.url);
    if (
      url.protocol !== 'http:' ||
      url.hostname !== '127.0.0.1' ||
      url.origin === location.origin ||
      url.username ||
      url.password ||
      url.search ||
      url.hash
    )
      throw new Error('Invalid isolated preview URL');
    const anchor = el('a', 'Open isolated Canvas2D preview');
    anchor.href = url.href;
    anchor.target = '_blank';
    anchor.rel = 'noopener noreferrer';
    root.append(
      identity(preview.source),
      el('p', `Canvas2D · ${preview.builtAt} · SHA-256 ${preview.sha256}`),
      anchor,
    );
  } catch {
    root.append(notice('Invalid preview descriptor; opening is disabled.'));
  }
  return root;
}
