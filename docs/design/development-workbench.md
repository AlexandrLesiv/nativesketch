# Native Sketch project workbench: interaction specification

Prepared 2026-10-16 for the lead's implementation plan. Read-only planning assignment;
no tracked sources, application features, desktop UI or system settings changed.
Sources: devtool-plan-notes, architecture/refactoring plan, quality/coverage,
quality/evidence-runner, quality/commands, current evidence types and task 024.

## Goal and scope

The solo developer should answer three questions without reconstructing repository state:
what has been built, what evidence supports it, and what can safely run on this host now?
They can read project documentation, navigate dependencies, inspect coverage and evidence,
start existing local build/check/run commands, watch jobs and open the web app preview.

Implement a project-local semantic HTML dashboard, launched from a Zed project task.
Zed retains editing, project search, terminal and debugger functions. The documented Zed
extension APIs do not presently establish a custom dashboard panel API; do not present a
browser tab as a Zed extension panel. A thin extension/MCP integration is deferred unless
the lead identifies a concrete need after the local task integration works. No new cloud
site, hosted telemetry, drawing-app controls, task editing, agent management or CI setup.

The same report components and versioned snapshot must support a static offline export.
The local runner service is an optional capability of that UI, not its data source of truth.
Static exports show actual snapshot time/source and disable run actions with a reason.

## Navigation and layout

Use a restrained developer-workbench appearance: system font, neutral surfaces, clear
borders, one accent for selection, native HTML form controls, monospace IDs/logs only.
System light/dark preference drives color scheme; do not change OS settings. Match visual
tokens across local and exported reports. Status is always text plus optional icon/color.

Persistent header: Native Sketch / Project workbench; short branch + commit; Dirty badge
when applicable; captured-at time; Local or Static report label; explicit Refresh snapshot.
Keep host/device eligibility visible near actions. Expose full hashes in a details panel.
Snapshot refresh reads metadata; it must not silently compile, enumerate tests, invoke IDEs
or trigger a job. If refreshing while a job runs, retain that job's starting source identity.

Desktop uses a 216–240 px navigation column and flexible content. Below 800 px the same
nav becomes a labeled disclosure above content. At 320 px/200% zoom, controls wrap, primary
actions remain reachable, tables scroll inside named regions, and no whole-page horizontal
scroll appears. Do not shrink text to fit a matrix. Inspector details flow below content on
small widths rather than covering rows. Browser Back restores filters and selected detail.

Seven destinations:

1. Overview: latest project snapshot, technical blockers and direct links to affected work.
2. Tasks: task list/dependency graph and task details.
3. Docs: repository documents, local links, headings and literal content search.
4. Dependencies: package inventory and architecture boundary relationships.
5. Coverage: feature/acceptance × platform evidence, plus individual attempts.
6. Actions & jobs: eligible existing commands and current/history jobs.
7. Metrics: qualified measurements, source-size debt and artifact sizes.

Web preview is an action/result within Actions & jobs, not an eighth information hierarchy.
Search is scoped to the active destination with a clear label. A global command palette is
not needed for the first implementation. Use stable URL route/filter parameters for links.

```text
┌ Native Sketch / Project workbench   branch · revision · Dirty   Captured… Refresh ┐
│ Overview        │ Tasks                     [Platform: All] [Search tasks…]       │
│ Tasks           │ [Table] [Dependency graph]                                     │
│ Docs            │ ID   Title              Board status   Technical evidence      │
│ Dependencies    │ 020  Skia integration   Waiting        Historical native pass  │
│ Coverage        │ 022  Host adapters      Waiting        Work in progress         │
│ Actions & jobs  │ ─ Selected task: 022 ─                                         │
│ Metrics         │ Scope · dependency links · acceptance gaps · source/evidence   │
└─────────────────┴────────────────────────────────────────────────────────────────┘
```

## Concrete screens and flows

### Overview

Show source identity first, then parallel facts: tasks awaiting owner acceptance; explicit
host/device gaps; evidence errors or stale results; source-cap violations; currently running
jobs. Counts name their denominator/scope and link to filtered tables. Do not display a
single completion/quality percentage, green "All good" banner or inferred release readiness.
An empty evidence folder means "No execution evidence loaded", not zero failures/pass.
The first action is View tasks or View coverage; Run checks stays an explicit secondary action.

### Tasks

Default table columns: ID/title, repository board status, owner role, dependency gate,
technical evidence summary, last referenced source. Sort/filter by platform, status, owner,
blocked dependency, text. Repository Status remains verbatim: waiting/review/done etc.
Separate technical-readiness annotations are labeled as derived and include provenance.
Never infer owner acceptance from passed tests, an implementation commit or a dependency's
technical readiness. Downstream preparation may be allowed while acceptance gates remain.

Selecting a task opens scope, owned paths, dependency list, acceptance IDs, evidence links,
document source and known limitations. "Open source file" uses a validated project path;
Copy path always works. No Mark done or mutable board controls in the initial scope.

Graph edges point prerequisite → dependent and have a visible legend. Graph nodes use the
same selection and status semantics as the table. Render a bounded SVG overview, with native
buttons for zoom/reset and focusable node links. A full Table view is always adjacent; each
row lists prerequisite/dependent links, so screen-reader users never need graph geometry.
Do not require drag, hover or color perception. On malformed/cyclic dependencies, show an
error and the affected IDs while preserving readable task content; no fabricated graph order.

### Docs

List architecture, product/design, quality/evidence, tasks and other project docs; use existing
folders rather than adding a second editorial taxonomy. Document view has title, source path,
modified/source identity, heading outline and rendered Markdown. Mermaid/diagrams may use
pre-rendered bounded SVG; always keep an equivalent adjacent text/list for architecture flows.
Search returns title/path and escaped literal text snippets. Highlight match without running
document HTML/scripts. Unsupported embeds show a clear placeholder and source text.
Relative links resolve within the snapshot project; missing links show "Not in this snapshot".
External links are visibly marked and require a normal intentional click; no auto-opening.

### Dependencies

Show inventory tabs by ecosystem with package, exact resolved version, direct/transitive,
scope/platform, available license metadata, source file and provenance. Missing metadata is
"Unknown"; do not label dependencies secure, current or licensed merely because they resolve.
Advisory/license check results, if installed, are separate dated evidence with tool identity.
Architecture boundary graph reuses the task graph's accessible list pattern. Distinguish a
declared allowed dependency, a resolved dependency and a reported violation; do not infer
Rust ownership policy from every package edge. Link to the actual checked-in policy.

### Coverage

Default is an acceptance table, not line coverage. Columns: case/feature, platform,
required level/input capability, automation state, execution state, source freshness,
environment validity, owning task. Filter platform, level, feature, status and text.
Each requirement is a distinct row; a passed core test never fills a native-E2E cell.
Native control AX inspection, VoiceOver speech, synthetic replay, actual OS event E2E and
physical device/latency evidence remain separately labeled input/evidence types.

Detail shows expected behavior, invariants, mapped executable IDs, discovery identity,
all execution attempts, subject/test binary hashes, toolchain/OS/device, environment reasons,
artifact links and explicit gaps. Show failed → passed retry as flaky at the relevant source;
retain the individual statuses. Never collapse stale pass into current pass.

State wording derives from existing evidence validators; UI must not reimplement precedence:
automation implemented/partial/planned/unsupported/not-applicable; execution passed/failed/
flaky/skipped/blocked/not run; metadata stale/missing-discovery/invalid-mapping/invalid-environment.
Where several apply, show independent columns or badges, not one ambiguous red/green dot.
"Report generated successfully" is a job outcome, not a coverage outcome. Hover tips are
optional; the explanation must also be available through keyboard-accessible details.

### Actions & jobs

Action catalog groups Build, Check, Prepare native tests, Run app, Execute native tests,
Preview web, and Export report. Reuse existing allowlisted entry points with fixed argument
schemas. Action cards show platform/profile/device requirements, expected foreground use,
command preview, artifact location and eligibility reason. No arbitrary shell editor.
Unsupported host/tool/device disables execution with a persistent explanation; copying an
exact command or opening a project file must not imply the command was verified on that host.

Background Build/Check/Prepare starts with one explicit Run click, no redundant confirmation.
Foreground Run app/native tests/benchmarks open a confirmation dialog naming the exact app,
device, desktop use and bounded reservation. User chooses duration and confirms "Reserve and
run" or Cancel. Offer reuse only for an active explicit reservation on this host whose
operation categories cover this action. Never infer authority from an old reservation after
service restart, expired time, a queued job or source-document text. No automatic default
device selection; display verified human-readable name plus stable ID. Enforce exclusive
foreground queue and revalidate the reservation/device/console before dequeue.

The dialog must distinguish a normal foreground reservation from an OS permission request.
If Accessibility/signing/account permission is absent, block with setup steps; do not alter
security settings, request permissions behind the user's back or retry indefinitely.

Job detail includes action, immutable start source, host/device, start/elapsed time, state,
bounded log tail, exit/result, artifact links and Cancel. Plain text logs escape markup and
control sequences. Auto-scroll pauses when the reader scrolls up, with "Jump to latest".
Do not make the entire log an aria-live region; announce state changes and new error summary.
Truncation has a visible byte/line marker and link to a retained bounded artifact if available.
Use phases reported by the command; absent progress uses an indeterminate indicator, no fake %.

Cancel requests process cleanup and displays Cancelling until confirmed. Completed work may
still exist; show Partial artifacts rather than claiming rollback. Retry creates a new job
and evidence attempt, preserves prior failure, and revalidates eligibility. Source changes
during execution show "Started at …; checkout now differs" and never rebind result identity.

### Web preview

Start only the existing production-build/serve route through its action. Display build
source/hash and served URL, then Open isolated preview. Optional embedded iframe must use a
separate origin with no runner token, privileged postMessage API or admin service worker scope.
Its browser-storage/offline state is clearly separate from the dashboard. Do not navigate
the privileged workbench origin to untrusted app content. Build failure leaves an existing
preview visibly labeled with its old identity; never label it as the failed build's preview.
Stop preview is a job control. Static reports show screenshot/link metadata but do not launch.

### Metrics

Group source cap/dependency gates, artifact/bundle sizes, test execution and performance.
Every metric has unit, source revision, build profile, device/toolchain where relevant,
fixture/protocol, capture time and evidence link. Compare only explicitly compatible series;
otherwise show side by side with "Not directly comparable" and the reason.
Separate compressed download, package/on-disk and linked binary size. Software input-to-submit,
GPU/presentation timestamps and physical pen-to-screen are distinct. Unknown or invalid samples
are not zero. No invented target/budget or winner from unrelated fixtures; show pending camera
measurements as pending. Accessible data tables accompany charts and use exact underlying data.

## State/error and accessibility requirements

| State | Presentation and allowed recovery |
| --- | --- |
| Loading snapshot | Stable page heading, busy region, last snapshot labeled while loading |
| No tasks/evidence/dependency metadata | Specific empty reason and relevant read/refresh action; no success badge |
| Malformed source/schema | Show parser/validator summary and safe path, preserve unaffected panels |
| Stale snapshot/results | Visible source comparison, explicit refresh/re-run action; history remains |
| Tool/device unavailable | Disabled action plus persistent exact reason and setup documentation |
| Queued/running | Visible host ownership, immutable source, bounded logs and Cancel |
| Permission or foreground lease expired | Block without opening UI; obtain a new explicit reservation |
| Failed/skipped/cancelled | Distinct text/status, diagnostic artifacts, retry as a new attempt |
| Service disconnected | Disable mutations, retain snapshot, indicate jobs' outcome unknown until reconciled |
| Static export | Persistent static/read-only badge; no tokens, executable commands or live job controls |

Use semantic nav/main/headings/table/caption/buttons/details/dialog elements, skip link,
visible keyboard focus and labeled filter controls. Dialog traps focus appropriately,
Escape cancels before start, closure returns focus to invoking action. Keyboard shortcuts
must not intercept typing in inputs; no single-letter global shortcuts needed. Native mouse,
keyboard and touch can operate everything; graph pan/drag and pen are optional conveniences.
Status changes are polite announcements; destructive/system permission operations are absent.
Meet WCAG 2.2 AA contrast and minimum targets, prefer 40–44 px action targets. Honor reduced
motion; no animated force graph. Test keyboard access and AX separately from screen-reader speech.

## Implementable acceptance IDs

| ID | Observable acceptance and required test |
| --- | --- |
| DX-NAV-001 | All seven destinations reachable by keyboard, URLs/Back preserve filters and selection; browser E2E |
| DX-SOURCE-001 | Header/detail show exact snapshot and job source; dirty/stale mismatch cannot display current pass; adapter tests + browser E2E |
| DX-TASK-001 | Board status and technical evidence remain separate; dependency table/graph select identical task and expose cycles; unit + keyboard E2E |
| DX-DOC-001 | Docs/search/links work offline; script/HTML/link payloads cannot execute or escape project; negative adapter/security tests + browser E2E |
| DX-DEP-001 | Resolved versions/provenance and unknown metadata honest; no invented advisory status; actual-lockfile fixtures + UI inspection |
| DX-COVERAGE-001 | Planned, missing-discovery, stale, skipped, invalid environment and flaky retry remain distinct; existing validator fixtures + browser E2E |
| DX-ACTION-001 | Only allowlisted actions/arguments run; unsupported host/device cannot start; service tests + disposable action E2E |
| DX-FOREGROUND-001 | No window focus from background actions; foreground requires valid exact-host reservation and device, rechecked on dequeue; service tests, one reserved native verification |
| DX-JOB-001 | Success/failure/cancel/output flood/source change produce bounded truthful history; process tests + fake disposable-job browser E2E |
| DX-PREVIEW-001 | Preview has separate origin/no admin capability; old preview stays identified after failure; isolation tests + actual Wasm browser smoke |
| DX-METRIC-001 | Units/identity/protocol retained, invalid data not zero, incompatible series not ranked; fixture tests + accessible table inspection |
| DX-EXPORT-001 | Static export opens offline with same docs/tasks/coverage/metrics and disabled actions; browser offline E2E |
| DX-ZED-001 | Project task starts workbench and exposes URL; stop cleans server; actual installed-Zed launch when permitted, otherwise explicit pending |
| DX-ACCESS-001 | Keyboard/focus, 320 px/reflow, light/dark/reduced motion, graph/table equivalence and labels; browser E2E + designer inspection; speech separately pending until tested |

Use stable IDs for rows/actions/jobs rather than DOM-position selectors. Reuse disposable
fixture projects and existing evidence fixtures. Keep real-job/native tests separate from
simulated service responses, and retain revisions, binary/tool identities, failures and retries.

## Implementation order and decisions

1. Freeze ProjectSnapshot, ActionSpec, Job and metric provenance contracts; adapt existing
   validators rather than introducing new acceptance inference. Build deterministic fixtures.
2. Implement read-only snapshot + semantic UI + offline static export; verify state fidelity.
3. Add bounded loopback job service and background actions with security/cancellation tests.
4. Add explicit foreground reservation/device eligibility, isolated preview and Zed task.
5. Run browser/native integration as applicable, designer review and fresh reviewer; preserve
   unavailable-host/Zed/speech gaps in final review packet.

Resolved recommendations: local dashboard rather than unsupported custom Zed panel; no task
mutation; no global shortcuts; no aggregate quality score; no auto-opening external IDEs;
background commands need no modal, foreground use needs a bounded reservation. These fit
the owner's approved developer-tool scope and need no additional product decision now.
Future task editing, agent transport, CI producer/publishing, remote hosts, shipping artifacts
and deeper Zed extension surfaces remain separate proposals, not incidental implementation.
