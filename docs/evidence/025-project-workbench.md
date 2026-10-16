# Project workbench: implementation and verification

The project GUI is implemented on the architecture branch. It reads authoritative tasks,
documents, dependency manifests and feature evidence; runs fixed background actions; opens
an isolated drawing preview; and exports the same interface as an offline HTML document.
Usage and deliberate limits are in the [workbench guide](../../apps/workbench/README.md).

This is downstream local work authorized while task 024 retains unavailable-host and
foreground acceptance gaps. It does not accept task 024, merge main or publish a release.
Zed integration consists of supported project tasks, not an invented custom-panel API.

## Recorded evidence

| Exact source | Result | Scope |
| --- | --- | --- |
| UI worker `530e6d2` | 10 headless fixture journeys passed; strict type/lint/format passed | Real built GUI with fixture data/responses; no real command execution |
| Service worker `4335bb5` | 16 service tests passed; fresh independent review cleared this scope | Bounded adapters, allowlist/origin/capability policy, queue, job failures and preview publication |
| Integrated `3d3e638` | 16 service and 12 browser journeys passed | Adds actual project export and production Wasm preview/drawing |
| Integrated `6b46fb4` | 17 service and 14 browser journeys passed | Aggregate coverage format, invalid-data presentation, dependency detail and retained actual preview identity |
| Integrated `6b46fb4` | Full background gate passed: 125 tooling tests, Rust checks/consumer tests, strict 133-file Swift lint, macOS build/pre-AppKit smoke and 17 GPU-free Swift tests | No native UI, desktop interaction or GPU benchmark |

The latest browser packet is `artifacts/evidence/025-workbench-reviewed`, with separate
backend discovery/results, browser discovery/results, raw Playwright report, process outcome,
identity, screenshots and an actual offline project report. At `6b46fb4`, workbench assets
SHA-256 are `0a809a6effab0c1d142355b033dce6acb616b0e842b747d8835ae67d35255822`;
headless Chromium SHA-256 is
`77d8278b1ac9d0687fad4e817ea83023ede09b1deef36e1864b49ba194ac9634`.
The raw Playwright attachment retains the isolated drawing build's source/hash and successful
job identity. That preview uses the current Canvas2D/Wasm baseline, not CanvasKit.

The 14 DX acceptance IDs are registered in `quality/features/workbench.json`. The scoped
report at `6b46fb4` has 23 platform/level rows: 21 passed, native Zed interaction and
screen-reader speech missing, and no mapping issues. This is scoped workbench evidence,
not all drawing-app functionality or line coverage. Later changes require refreshed evidence.

## Review findings and repairs

Independent review repaired cancellation during preview finalization, source validation
before publishing, bounded aggregate coverage input and toolchain auto-install during
read-only inventory. Designer review repaired narrow tables, buried selected details and
job focus/log state. Actual-project review then found screenshot/navigation synchronization,
overlong dependency rows and misleading zero/no-match wording for invalid coverage; these
were repaired and covered by the `6b46fb4` journeys. All seven actual-project captures were
reviewed. A final readability follow-up `7f7dfb8` collapses detailed mapping diagnostics while
preserving the prominent stale/partial warning and all diagnostic entries.

The first cancellation repair (`aafde17`, integrated `a6c5650`) stops delayed Node workers
and rejects pre-aborted evidence. Fresh review reproduced a deeper gap: a nested detached
job could survive cancellation of its parent runner. The same topology affects a workbench
job invoking another repository process helper. This is a correctness finding despite the
passing happy-path checks above. Explicit nested process ownership is being repaired;
this packet does not yet claim final cancellation acceptance or task completion.

## Remaining acceptance and scope

- The nested-process repair requires fresh review and background regression execution.
- Native Zed task launch and screen-reader speech remain unexecuted. No Finder, CUA,
  native app, simulator or desktop permission flow was used for these workbench checks.
- Foreground actions are disabled server-side and in the interface. No UI control can
  enable native tests, app launches, GPU benchmarks or IDE opening in this version.
- Windows process ownership and other host/browser combinations are not certified by
  macOS/headless Chromium results. Missing detailed SDK prerequisites can fail a build.
- Markdown diagrams remain readable source; dependency inventory is not a security audit.
  Current coverage reads one explicit evidence pair and does not search for favorable history.
- Task 024 platform, physical-input, packaging and native presentation gaps remain in the
  [architecture acceptance packet](024-background-acceptance.md). No CI rollout or release.
