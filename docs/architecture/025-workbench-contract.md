# Workbench contract, version 1

Lead implementation contract for [task 025](../../tasks/025-project-workbench.md), following
the [plan](../development-workbench-plan.md) and designer's interaction specification.
This freezes the boundary before service and interface writers start. It does not claim
that task 024's unavailable host checks or remaining platform migrations are complete.

## Ownership

The service worker owns `scripts/workbench/`, except the pure shared contracts after their
initial freeze. The interface worker owns `apps/workbench/`. The lead owns root tooling,
`.zed/tasks.json`, integration, evidence and task documentation. Both workers use separate
worktrees from the same frozen integration commit. One owner changes a shared contract;
notify the other writer before altering a field or endpoint. No recursive delegation.

Pure, serializable shared types live in `scripts/workbench/contracts/`. They may import
existing evidence types using `import type`; they cannot import filesystem, process or
server code at runtime. Browser views consume these types and the same pure view modules
in local and static modes. No new framework or cloud service is required.

## Snapshot

`ProjectSnapshot` has `schemaVersion: 1`, `generatedAt` (ISO), `source` (existing revision
and dirty-diff SHA-256), `branch` (string), `host` (platform/architecture), and the sections
below. Do not include a home-directory path, live capability, environment variables, or
process object. Paths in report data are repository-relative identifiers.

Each section has `state: ready | partial | missing | invalid`, `diagnostics` and `data`.
A diagnostic has severity, stable code, safe relative source path when applicable, and
plain-text message. A failed parser produces an invalid/partial section with visible
diagnostics; it cannot become an empty successful section. Other sections remain usable.

| Section | Data and authority |
| --- | --- |
| tasks | Existing board Task fields, unchanged status/roles/dependencies/brief. A task may have safe readable fields while its section is invalid; never infer ready/done or invent graph order for a cycle. |
| documents | `{id, title, format, text, sha256}`; id is an indexed relative path, format markdown or text. Search/render from this bounded snapshot, including offline. |
| dependencies | Rows with stable id, ecosystem, package name, version or null, resolved/declared/unknown version kind, direct/transitive/unknown scope, license or null, provenance path, and dependency IDs. Separate actual advisory evidence from inventory. |
| coverage | Existing validated report rows plus ownerTask/invariants joined by case ID, discovery source or null, source freshness current/stale/unknown, and mapping issues. Retain individual attempts and artifact IDs. |
| metrics | Rows with stable id, label, value or null, unit, status measured/missing/invalid, source identity, capture time, scope, configuration and evidence paths. No numeric value for missing data. |
| actions | Serializable action catalog entries: id, label, category, platform, fixed command preview, foreground flag, enabled flag, disabled reason, and prerequisites. No capability/token. |

Overview is derived presentation of these sections, not a second quality model. Jobs are
local service state outside the snapshot. Unknown evidence does not mean zero failures.

Reuse `readBoard`, evidence validators and `buildReport`. For no discovery/results, render
the registered requirements as missing discovery/not run without inventing an empty
successful report. If a real report is loaded, its existing status precedence remains
authoritative. Keep report-source freshness separate: an old passing attempt is never a
current-source pass. Invalid mapping, failed, skipped, flaky and environment errors stay
visible, and a report-producing process exiting zero is not application acceptance.

Read current `artifacts/evidence/discovery.json` and `results.json` as the initial evidence
pair; do not search arbitrary historical directories for a more favorable result. Display
the exact loaded paths/source and all recorded attempts. Historical evidence remains linked
from repository docs. A later user-selectable history browser is outside this initial slice.

## Bounded repository adapters

Index approved tracked Markdown/text documents under docs/tasks and project readmes, plus
root README and AGENTS. Do not follow arbitrary links into the filesystem. Every read uses
root containment and realpath checks; reject symlink escapes. Limit documents to 1,000,
2 MiB each and 12 MiB total; snapshot serialized size to 24 MiB. Exceeding a bound produces
a visible diagnostic, not silent omission. Ignore build caches, dependencies and secrets.

Rust inventory uses locked offline Cargo metadata with a timeout; npm uses its lockfile.
Additional ecosystems may show declared data with provenance or an explicit unavailable
inventory, never fictional resolved versions. Source metrics reuse the source policy and
physical-line counting rules, including nonignored untracked source. Bound that scan too.
Package/performance metrics require actual saved evidence with compatible provenance; no
parsing arbitrary prose into a claimed measurement or ranking incompatible protocols.

Capture source identity before and after collection. If it changes, reject that refresh
and retain the previous snapshot labeled stale. Refresh does not build, enumerate tests,
start an IDE or obtain permissions. Background metadata tools are read-only and offline.

## Local HTTP boundary

Bind `127.0.0.1` only on an available port; print the URL, never auto-open it. Use exact Host
validation, no CORS, no reflected Origin, CSP, no-referrer policy and frame restrictions.
Serve only the built workbench assets and explicit JSON endpoints, not the repository root.
The ephemeral action capability is delivered only to the local UI and kept in memory. It
must not appear in URLs, access logs, snapshots, static exports or drawing-app preview.

All mutations require same-origin requests and the capability header. Reject unexpected
methods/content types, oversized bodies, unknown fields, action IDs, paths and parameters.
Never execute browser-provided shell text, argv, environment variables or executable paths.

| Endpoint | Contract |
| --- | --- |
| `GET /api/session` | Local bootstrap capability plus service policy; same-origin browser context only, never preview/export data. |
| `GET /api/snapshot` | Last valid ProjectSnapshot, or explicit unavailable response during initial failure. |
| `POST /api/refresh` | Recollect through bounded adapters; return snapshot or diagnostic failure without replacing the last valid snapshot. |
| `GET /api/jobs` | Bounded job records, exact start source, current state and bounded plain-text log tails. |
| `POST /api/jobs` | Body `{actionId}` only in version 1. Fixed catalog expands argv; eligibility rechecked at dequeue. Return job or explicit rejection. |
| `POST /api/jobs/{id}/cancel` | Cancel only a known owned job; preserve terminal outcome, report Cancelling until cleanup completes. |
| `GET /api/preview` | Current isolated preview descriptor or missing reason; descriptor includes build source/hash and URL, no admin token. |
| `GET /api/artifacts/{id}` | Only indexed evidence IDs with bounded root-confined reads; content is plain text or safe download, never active HTML on the admin origin. |

HTTP reads that expose repository data also require the local session capability after
bootstrap. Static asset/bootstrap requests must reject foreign browser origins/fetch sites;
no unauthenticated cross-origin JSON endpoint or DNS-rebinding hostname is accepted.

## Actions, process ownership and preview

Catalog commands reuse existing Node entry points via executable/argv arrays. Displayed
command previews come from the same definitions. Start with fixed debug/release macOS,
Apple simulator, Android, Linux, Windows and web/Wasm build entries; background checks,
native-test preparation, web preview and snapshot export. Unsupported tools/hosts have
specific disabled reasons. Do not expose merging, publishing, installations or global setup.

The current service policy has `foregroundExecution: false`. **All app launches, native UI
tests, GPU benchmarks and IDE opening are disabled on the server**, including direct API
requests. Display a persistent explanation; do not show a reservation/permission dialog or
infer permission from the prior desktop grant. A future explicit foreground session can be
added separately after this restriction; no unattended native action is enabled now.

One active job per checkout, at most four queued and thirty retained records. Each job has
id/action/state, immutable start source, queued/start/end times, exact command preview,
exit code/signal/reason, bounded plain-text output and artifact IDs. Output is at most 1 MiB
per job; excess output terminates the job with a visible failure, not a success with missing
diagnostics. Limit duration per action. Cancel/shutdown terminates and reaps the owned
process group with a bounded graceful period. Never kill unrelated processes. Windows job
execution stays disabled until equivalent process-tree cleanup is verified on Windows.

Retry is a new job; prior attempts remain. Source changes during execution are labeled
explicitly and cannot rebind a build's identity. Child exit/timeout/cancellation dominates
any earlier success-looking log. Native preparation's blocked evidence remains blocked.

Web preview first builds the actual existing Wasm app, then copies the successful output
into an immutable bounded preview directory. Serve it on a separate loopback origin, with
no admin capability or privileged messages. A failed rebuild preserves an older preview
under its old identity. Do not navigate the privileged workbench origin to app HTML. The
initial preview is the current Canvas2D app; do not label it CanvasKit until migrated.

## Interface/export/build integration

The UI has seven destinations and URL-based filters/selection with browser Back behavior.
Use semantic tables/details/forms, safe text-only Markdown rendering, validated links and
an SVG task graph with equivalent dependency lists. Raw HTML, SVG scripts and document
JavaScript cannot execute. No external assets are needed for local or exported use.

Use Vite for one browser bundle and shared CSS. Offline export embeds that exact bundle,
styles and escaped snapshot JSON into a self-contained HTML file. It works at file URLs
with network disabled, uses the same seven views, and permanently disables job/refresh/
preview execution. Never serialize service capabilities or the live jobs manager.

The command family is `node scripts/workbench.ts serve|snapshot|export` with explicit
artifact output paths for snapshot/export. Zed tasks invoke these project entry points;
no extension installation, fake custom panel, automatic IDE launch or CI workflow is needed.

Required checks: malformed/escaping/oversized/source-change adapter fixtures; Host/Origin/
CSRF/allowlist/foreground rejection; real disposable child failure/cancel/flood cleanup;
real built-UI keyboard/navigation/filter/docs-safety/job/offline-export browser journeys;
isolated actual Wasm preview smoke; designer headless screenshot review and fresh independent
review. Register DX acceptance IDs, exact source/bundle/tool identity and remaining native,
Zed-app and screen-reader speech gaps. All handwritten files stay within 200 lines.
