# Native Sketch

A macOS-first vector sketching application with a shared Rust engine and native UI.
This repository is the **project and agent-workflow foundation**, not a finished drawing app.

## What works now

- Dependency-free Rust workspace with a core crate and a small C ABI boundary.
- Native AppKit application shell that links to Rust and checks the ABI version.
- Local build/check commands and a headless Swift-to-Rust integration smoke check.
- Project-local GPT-6 Astra roles, design and quality standards, and a ready task backlog.

Drawing, renderer integration, selection, undo, and document saving are next milestones.
Windows is planned as C# + WinUI 3; no Windows application is implemented yet.

## Start in Codex

Open this folder as a local project. Project configuration may require Codex's normal
trust confirmation when opening a new repository. Select GPT-6 Astra if an existing chat
has a different model; project defaults do not replace explicit chat overrides.

The primary agent is the lead. Seven specialist configurations are in `.codex/agents/`.
Use this request to begin the first feature:

> Follow AGENTS.md and tasks/001-first-stroke.md. Start with ui_ux_designer to define
> the first macOS drawing interaction. Delegate the independent Rust and AppKit work
> after agreeing the input contract. Use isolated worktrees for concurrent writers.
> Implement and test the milestone, obtain a fresh review, and bring me a running app
> and evidence for merge approval. Keep the renderer decision explicit.

Nothing runs on a schedule. Agents are invoked by a project task. Role files give
instructions, not independent permission boundaries. See docs/workflow.md.

## Local development

Prerequisites: Rust 1.85+ with cargo, rustfmt, and clippy; Swift 6+ and a macOS SDK;
Python 3.9+ for the small task-board checks. macOS 14 is the provisional shell minimum.
The scaffold has no external Rust or Swift package dependencies.

```sh
./scripts/check            # metadata, Rust checks, Swift build and ABI smoke check
./scripts/run-macos        # build a development .app bundle and open it
./scripts/board            # show ready tasks and dependencies
./scripts/check --core     # metadata and Rust checks only; supports non-Mac hosts
```

Generated build output and local evidence belong under ignored `target/`, `.build-cache/`,
and `artifacts/`. Do not use personal documents as automated-test fixtures.

## Read as needed

- [Product scope](docs/product.md)
- [Architecture](docs/architecture.md)
- [UI/UX principles](docs/design/principles.md)
- [Agent workflow](docs/workflow.md)
- [Quality gates](docs/quality.md)
- [Foundation validation](docs/setup-validation.md)
- [Tasks](tasks/README.md)

No cloud repository, CI provider, release identity, or open-source license is selected.
Local setup does not enable remote branch protection or validate physical pen latency.

