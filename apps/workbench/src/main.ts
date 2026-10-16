import type {
  Bootstrap,
  Job,
  PreviewResponse,
  ProjectSnapshot,
} from '../../../scripts/workbench/contracts/index.ts';
import { actionsView } from './actions.ts';
import { coverageView } from './coverage.ts';
import { dependenciesView, metricsView } from './inventory.ts';
import { docsView } from './docs.ts';
import { button, el } from './dom.ts';
import { overviewView } from './overview.ts';
import { currentView, route } from './route.ts';
import { LocalService } from './service.ts';
import { preserveView } from './view-state.ts';
import { shell } from './shell.ts';
import { tasksView } from './tasks.ts';
import './styles.css';
import './layout.css';
const mount = document.getElementById('workbench-root');
const bootstrapText = document.getElementById('workbench-bootstrap')?.textContent;
if (!mount || !bootstrapText) throw new Error('Workbench bootstrap is missing');
const root = mount;
let snapshot: ProjectSnapshot | null = null;
let jobs: Job[] = [];
let preview: PreviewResponse | null = null;
let message = '';
let live = false;
let pending = false;
let focusJob: string | null = null;
const service = new LocalService();
function render(focus = false): void {
  if (!snapshot) {
    root.replaceChildren(el('h1', 'Project workbench'), el('p', message || 'Loading snapshot…'));
    if (live)
      root.append(
        button('Reconnect service', () => {
          void refresh();
        }),
      );
    return;
  }
  const page = shell(snapshot, live, !pending, () => {
    void refresh();
  });
  const context = {
    live,
    connected: service.connected && !pending,
    jobs,
    preview,
    start: (id: string) => {
      void mutate(async () => {
        const result = await service.start(id);
        focusJob = result.job.id;
      });
    },
    artifact: (id: string) => {
      void mutate(() => service.artifact(id));
    },
    cancel: (id: string) => {
      void mutate(() => service.cancel(id));
    },
  };
  const views = {
    overview: overviewView,
    tasks: tasksView,
    docs: docsView,
    dependencies: dependenciesView,
    coverage: coverageView,
    metrics: metricsView,
  };
  const view = currentView();
  page.main.append(view === 'actions' ? actionsView(snapshot, context) : views[view](snapshot));
  page.status.textContent = message;
  root.replaceChildren(page.root);
  if (focus) {
    const selected = route().has('selected') ? document.getElementById('selected-detail') : null;
    (selected ?? page.main).focus();
    selected?.scrollIntoView({ block: 'start' });
  }
  const heading = route().get('heading');
  if (heading) document.getElementById(`doc-heading-${heading}`)?.scrollIntoView();
}
async function updateJobs(): Promise<void> {
  jobs = await service.jobs();
  preview = await service.preview();
}
function failure(error: unknown): void {
  service.connected = false;
  message = `Service unavailable: ${error instanceof Error ? error.message : 'Unknown error'}. Last snapshot retained; job outcomes require reconciliation.`;
}
async function refresh(): Promise<void> {
  if (!live || pending) return;
  pending = true;
  message = 'Refreshing metadata…';
  render();
  try {
    if (!service.connected) await service.connect();
    const result = await service.snapshot(true);
    snapshot = result.snapshot;
    message = `${result.stale ? 'Snapshot is stale. ' : 'Snapshot refreshed. '}${result.diagnostics.map((item) => item.message).join(' ')}`;
    await updateJobs();
  } catch (error) {
    failure(error);
  } finally {
    pending = false;
    render();
  }
}
async function mutate(action: () => Promise<unknown>): Promise<void> {
  if (!live || !service.connected || pending) return;
  pending = true;
  try {
    await action();
    await updateJobs();
    message = 'Job state updated. Process status is not application acceptance.';
  } catch (error) {
    failure(error);
  } finally {
    pending = false;
    preserveView(root, () => render());
    if (focusJob) {
      const job = [...root.querySelectorAll<HTMLElement>('[data-job]')].find(
        (node) => node.dataset.job === focusJob,
      );
      job?.focus();
      job?.scrollIntoView({ block: 'start' });
      focusJob = null;
    }
  }
}
async function start(): Promise<void> {
  try {
    const bootstrap = JSON.parse(bootstrapText ?? '') as Bootstrap;
    if (bootstrap.mode === 'static') {
      snapshot = bootstrap.snapshot;
      message = 'Offline snapshot. All execution is disabled.';
    } else if (bootstrap.mode === 'live') {
      live = true;
      await service.connect();
      const result = await service.snapshot();
      snapshot = result.snapshot;
      message = `${result.stale ? 'Snapshot is stale. ' : ''}${result.diagnostics.map((item) => item.message).join(' ')}`;
      await updateJobs();
    } else throw new Error('Unsupported bootstrap mode');
    if (snapshot.schemaVersion !== 1) throw new Error('Unsupported snapshot schema');
  } catch (error) {
    failure(error);
  }
  render();
  if (live)
    setInterval(() => {
      void poll();
    }, 3000);
}
async function poll(): Promise<void> {
  if (pending || !service.connected || currentView() !== 'actions') return;
  if (
    document.activeElement instanceof HTMLInputElement ||
    document.activeElement instanceof HTMLSelectElement
  )
    return;
  try {
    const previous = JSON.stringify([jobs, preview]);
    await updateJobs();
    if (previous !== JSON.stringify([jobs, preview])) preserveView(root, () => render());
  } catch (error) {
    failure(error);
    render();
  }
}
window.addEventListener('hashchange', () => render(true));
void start();
