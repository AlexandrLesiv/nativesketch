# Native Sketch

Build a responsive, offline-first, editable vector sketching app. The owner is a solo
developer who wants agents to implement and test autonomously, then present a complete
change for human merge approval. Native Sketch is a working name.

## Product and architecture

- macOS first: Swift with actual AppKit controls wherever applicable; custom canvas.
- Shared drawing, document, editing, and persistence semantics belong in Rust.
- Windows: C# + WinUI 3. Future: UIKit on iOS/iPadOS, Kotlin/Views on Android,
  GTK4/libadwaita on Linux, semantic HTML + Rust/Wasm on web.
- Skia is the first benchmark baseline; Vello is a candidate. Renderer selection is OPEN.
- Maximum drawing responsiveness matters. Performance claims require measurements.
- Budget, reference hardware, product name, and distribution targets remain undecided.
- Use docs/product.md for scope; docs/architecture.md for boundaries;
  docs/design/principles.md for UI work; docs/quality.md for applicable evidence.
  Read only what the current change needs.

## Agent workflow

The primary agent acts as lead/integrator. Use the configured specialists for scoped
work when it saves time or improves quality. Delegate UI interaction design and review
to ui_ux_designer for substantial visible features. Use a fresh reviewer for substantive
changes; do not claim independent review unless it actually ran.

- At most three subagents concurrently and two implementation writers; one feature in flight.
- Give each worker an outcome, acceptance cases, starting commit, owned paths,
  dependencies, relevant contracts, required evidence, and stopping condition.
- Agree shared interfaces before parallel implementation. One owner changes a contract.
- Use isolated worktrees for concurrent writers. Subagents do NOT automatically get
  their own worktree. If isolation is unavailable, serialize edits or explicitly assign
  disjoint paths. Do not run worktree creation from a worktree recursively.
- Specialist workers report to the primary; do not delegate recursively without a
  specific independent task and available capacity.
- Run GPU performance measurements and native UI automation exclusively on their host.
- After two unproductive repair cycles, the lead diagnoses or reduces scope before retrying.

## Completion and authority

Implement, run, inspect, and fix the authorized work through to a reviewable result.
Routine local builds and tests use disposable fixtures: run and repair them without
asking at every step. Respect current host permissions; do not change global Codex
settings or grant trust to other folders.

Local task-branch commits are permitted. The owner approves merging into main and
publishing or distributing releases. Keep implementation and checks complete before
requesting that approval. This is a workflow rule, not a configured branch-protection
service; no remote or server-enforced merge policy exists yet.

Run ./scripts/check for scaffold/integration changes; use narrower relevant checks
for low-impact edits. State what passed, failed, and was not run. Evidence must identify
the tested revision. Do not alter baselines or thresholds merely to get green checks.
Do not describe the scaffold as a working drawing app.

