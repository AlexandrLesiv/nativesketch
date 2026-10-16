# Native Sketch project workbench

A local GUI for the repository's tasks, documents, dependencies, feature verification,
background build/check jobs and qualified metrics. The same view modules produce a
single-file offline report. This is a development tool, separate from the drawing app.

## Start

Use the repository's pinned Node 24.22.0 and installed root dependencies:

```sh
npm run workbench:build
npm run workbench
```

The service prints its loopback URL. Open that URL in your browser when convenient;
the command does not open a browser, Finder, IDE or native app. Stop with Ctrl+C.
An optional fixed port is available through `node scripts/workbench.ts serve --port 8085`.
Only one service may own a checkout. Do not remove an active service's ownership lock.

In Zed's task picker, use **Native Sketch: build project workbench**, then
**Native Sketch: start project workbench (prints URL)**. Background check and web-build
tasks are also provided in `.zed/tasks.json`. Tasks do not automatically reveal a terminal
or save edited files. Their Node executable must resolve to the pinned version in Zed's
environment. Actual interaction with the installed Zed app remains unverified.
This is project-task integration, not a custom Zed extension panel.

## Export and evidence

```sh
node scripts/workbench.ts snapshot --output artifacts/workbench/snapshot-NEW.json
node scripts/workbench.ts export --output artifacts/workbench/report-NEW.html
node scripts/test-workbench.ts --output artifacts/evidence/workbench-NEW
```

Choose a new output filename/directory each time; existing reports are never overwritten.
The test command requires `SKETCH_BROWSER_EXECUTABLE` pointing to an installed headless
Chromium executable. It does not download or launch a desktop browser. Tests preserve
discovery, exact source/assets/browser identity, backend results, browser attempts and
screenshots. Real-service tests disable network traces to avoid recording session tokens.
Fixture response tests, real background jobs and actual Wasm drawing are separately named.

The GUI reads current `artifacts/evidence/discovery.json` and `results.json`, preserving
their recorded source and all attempts. It does not search historical folders for a
favorable result. Other retained evidence is reachable through the repository's evidence
documents; it is not automatically promoted into the current coverage matrix.

## Deliberate limits

- All foreground actions are disabled in both the service and interface. No app launch,
  native UI test, GPU benchmark, IDE-opening or permission-grant action is enabled.
- Fixed background commands run one at a time, with bounded queues, logs and duration.
  Process exit success is a job result, not proof that a platform's acceptance passed.
- Builds use existing installed toolchains. Inventory refresh never installs a Rust toolchain.
  A visible action can still fail if its SDK or other detailed prerequisite is absent.
- Jobs are local to this service instance; retained artifacts outlive its bounded job list.
  Windows execution is disabled until native process-tree cleanup is verified.
- The isolated drawing preview uses CanvasKit 0.42.0 with the portable Rust/Wasm app.
  The workbench itself uses semantic HTML and does not need Skia.
- Markdown code fences, including Mermaid, remain readable source; unsupported embedded
  content is not executed. Package metadata is inventory, not an advisory/license audit.
- Native Zed launch, screen-reader speech and other OS/browser combinations are pending.
  No CI producer, hosting, release publication or task-status editing is implemented.

See the [plan](../../docs/development-workbench-plan.md),
[contract](../../docs/architecture/025-workbench-contract.md) and
[acceptance registry](../../quality/features/workbench.json).
