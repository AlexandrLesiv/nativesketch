# Native Sketch

Build a responsive, offline-first, editable vector sketching app. The owner is a solo
developer who wants agents to implement and test autonomously, then present a complete
change for human merge approval. Native Sketch is a working name.

## Product and architecture

- macOS first: Swift with actual AppKit controls wherever applicable; custom canvas.
- Shared drawing, document, editing, and persistence semantics belong in Rust.
- Windows: C# + WinUI 3. Future: UIKit on iOS/iPadOS, Kotlin/Views on Android,
  GTK4/libadwaita on Linux, semantic HTML + Rust/Wasm on web.
- Skia is selected, including CanvasKit on web. Retain historical comparisons; do not
  reopen renderer selection or maintain production Vello support without new scope.
- Maximum drawing responsiveness matters. Performance claims require measurements.
- Budget, reference hardware, product name, and distribution targets remain undecided.
- Use docs/product.md for scope; docs/architecture.md for boundaries;
  docs/design/principles.md for UI work; docs/quality.md for applicable evidence.
  Read only what the current change needs. docs/refactoring-plan.md orders the migration;
  docs/quality/coverage.md defines acceptance-to-test traceability.

## Architecture and quality gates

- Handwritten source files, including tests and scripts, must contain at most 200 physical
  lines after formatting (blank lines/comments count). Generated/vendor code, lockfiles
  and data are excluded explicitly. Existing violations are migration debt, not exemptions
  for new work: new/extracted files comply; an oversized touched file must not grow and
  must have a scoped extraction task. The completed refactor permits no source violations.
  Do not compress statements or split one coupled object into arbitrary numbered files.
- Share tool/action IDs, validated settings, availability and editing behavior in Rust;
  retain native controls, layout, focus, localization and accessibility in each host.
- Use composition, typed commands, explicit interaction states and narrow platform
  adapters. Keep document, session, viewport, native presentation and disposable caches
  separate. Native widgets and Skia objects must not enter the document model.
- One execution owner mutates an editor. Define buffer ownership, cancellation, revisions
  and stale-result rejection before asynchronous work. No disk/GPU waits on the ink path.
- Define future agent commands/queries now; do not implement agent transport or permissions
  incidentally. Real samples are ordered; predictions remain temporary; commit once per undo.
- Route system shortcuts, native controls and future agent signals into the same typed
  command dispatcher. Adapters translate intent; they must not duplicate editing or saving
  behavior. Native field editing keeps first-responder priority; dialogs are native effects.
- New repository automation uses native Node.js TypeScript `.ts` ES modules, Oxlint/Oxfmt
  and strict erasable types. Use explicit `.ts` imports and `import type`; no transpiler,
  runtime enums, parameter properties, decorators, `any` casts or type-check suppressions.
  Migrate owned shell/PowerShell and routine Python orchestration in task 016. Until then,
  run existing scripts; do not delete working checks early. Preserve Python benchmark
  analysis until replacement equivalence is demonstrated. Native build tools remain required.
- Each behavioral change names acceptance/invariant IDs and adds relevant tests. Native
  feature acceptance requires real UI E2E on the applicable host, separately from core,
  bridge and injected replay checks. Missing hardware/unsupported automation is a coverage
  gap, never a pass. Use disposable fixtures and stable accessibility identifiers.
- Record exact revision/build, platform/device, test IDs, results and artifacts. Separate
  automation existence from execution status; retain skips, failures and flaky retries.
- Instrument operation/failure boundaries with structured, correlated, redacted diagnostics.
  No per-sample production logging, swallowed errors or unbounded queues/logs/caches.
  Instrument native intent origins and shared command outcomes/state transitions centrally;
  preserve request/session/operation correlation across workers. Use allowlisted metadata,
  never document text/paths/credentials. Bounded queues drain off input callbacks; count
  drops and sink failures without failing edits. Test redaction, rejection and overload.
  Follow docs/architecture/022-typed-bindings-migration.md and docs/quality/agent-workflow.md.
- Typed FFI with generated declarations replaces the custom packet protocol as the target
  language boundary. Preserve shared Rust commands/behavior and working consumers until
  parity passes; document encoding is not the command transport. No incidental executor,
  renderer or UniFFI adoption. The migration plan is not evidence of implementation.
- Run applicable format/lint/type/dependency/size checks and behavior/E2E/recovery tests.
  Planned tooling is not an installed gate; docs/quality.md distinguishes current checks
  from tasks that install them. Never weaken a baseline, exclusion or test to obtain green.

## Agent workflow

The primary agent acts as lead/integrator. Use the configured specialists for scoped
work when it saves time or improves quality. Delegate UI interaction design and review
to ui_ux_designer for substantial visible features. Use a fresh reviewer for substantive
changes; do not claim independent review unless it actually ran.

- At most three subagents concurrently and two implementation writers; one feature in flight.
- Give each worker an outcome, acceptance cases, starting commit, owned paths,
  dependencies, relevant contracts, required evidence, and stopping condition.
- Agree shared interfaces before parallel implementation. One owner changes a contract.
- Lead plus rust_engine own architecture planning; ui_ux_designer reviews native behavior.
  ui_ux_designer coordinates shared product design and per-platform Figma variants/reviews;
  owner approval remains required before new pages/tools are implemented. Use macos_ui for
  AppKit, ios_ui for UIKit iPhone/iPad and assigned visionOS 2D, android_ui for Kotlin/Views
  and Android-on-ChromeOS, windows_ui for WinUI. Assign Linux/web and Apple viewer work
  explicitly; one owner for shared Apple files. Designers request specialist reviews through
  the lead, not recursive delegation. Adding roles does not raise concurrency limits.
  build_reliability owns quality runners, coverage reporting and diagnostics infrastructure;
  implementers own feature tests/logging; rendering_performance owns measurement validity;
  reviewer independently checks contracts and evidence. No extra permanent quality agent.
  Follow docs/quality/agent-workflow.md: small vertical slices, explicit old-path retirement,
  and focused maintainability/security/performance/design reviews by existing specialists.
- Use isolated worktrees for concurrent writers. Subagents do NOT automatically get
  their own worktree. If isolation is unavailable, serialize edits or explicitly assign
  disjoint paths. Do not run worktree creation from a worktree recursively.
- Specialist workers report to the primary; do not delegate recursively without a
  specific independent task and available capacity.
- Run normal builds, lint, core/bridge tests and evidence processing in the background without
  opening or focusing apps. Native UI automation and GPU measurements need an explicitly
  reserved foreground session on their host, or a dedicated host; do not steal user focus.
  Run native UI tests when verifying a relevant commit or developing UI tests, not on every
  background check. Native E2E defaults to preparation only; use --interactive in that session.
  Hidden/minimized/off-console runs cannot substitute for visible native acceptance.
- After two unproductive repair cycles, the lead diagnoses or reduces scope before retrying.

## Completion and authority

Explicit owner review is required BEFORE implementing material architecture choices:
new/custom protocols, serialization/document formats, language/runtime boundaries, mutable
state ownership or thread topology, renderer/framework replacements, and security/trust
models. Present concrete interfaces, alternatives, risks, migration/removal and test plans;
record the owner's approval. Lead/agent agreement or general refactoring authorization is
not a substitute. Existing explicit approval covers routine work within its scope; do not
re-ask for ordinary fixes/extraction/tests. Material deviations require renewed review.
Continue independent authorized work while waiting. The typed-binding direction is approved;
task 022A requests owner review of the concrete contract before implementation.

New user-facing features and extensions to additional platforms require explicit owner
approval before implementation. A general implementation/refactoring request, an agent-authored
design, platform convention or test convenience is not approval for extra features. Propose
the scope and obtain approval; continue already authorized fixes and verification meanwhile.
Retain the existing macOS color picker and other already implemented color controls. Do not
add a color picker to another platform or expand these controls without explicit approval.
For new application pages/screens or tools, ui_ux_designer must prepare a Figma design
and obtain explicit owner approval of an identified version before implementation. Follow
docs/design/principles.md for native feasibility, handoff, approval records and material
deviations. Missing Figma access is a blocker for that UI slice, not permission to bypass
review; continue independent authorized work. Routine fixes within approved designs proceed.

Implement, run, inspect, and fix the authorized work through to a reviewable result.
Routine local builds and tests use disposable fixtures: run and repair them without
asking at every step. Respect current host permissions; do not change global Codex
settings or grant trust to other folders.

Local task-branch commits are permitted. The owner approves merging into main and
publishing or distributing releases. Keep implementation and checks complete before
requesting that approval. This is a workflow rule, not a configured branch-protection
service; no remote or server-enforced merge policy exists yet.

Run node scripts/check.ts for scaffold/integration changes after npm ci --ignore-scripts;
use narrower relevant checks from docs/quality/commands.md for low-impact edits. Retained
legacy scripts remain available until each host's parity is verified. State what passed,
failed, and was not run. Evidence must identify
the tested revision. Do not alter baselines or thresholds merely to get green checks.
Do not describe the scaffold as a working drawing app.

