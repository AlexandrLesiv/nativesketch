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
| Integrated `0aa3e00` | Full background gate passed: 135 tooling tests, Rust workspace/C/Wasm consumers, strict 133-file Swift lint, macOS build/pre-AppKit smoke and 17 GPU-free Swift tests | Includes nested automation ownership; no desktop interaction |
| Integrated `0aa3e00`, serial packet | 17 service and 14 browser journeys passed; clean source throughout | Actual project export, isolated production Wasm drawing and all seven workbench views |

The latest browser packet is `artifacts/evidence/025-workbench-0aa3e00-serial`, with separate
backend discovery/results, browser discovery/results, raw Playwright report, process outcome,
identity, screenshots and an actual offline project report. At `0aa3e00`, workbench assets
SHA-256 are `5c5bf709447f2c0c13db652dea462f793579cc69c426c00d64abfbec139e3a06`;
headless Chromium SHA-256 is
`77d8278b1ac9d0687fad4e817ea83023ede09b1deef36e1864b49ba194ac9634`.
The raw Playwright attachment retains the isolated drawing build's source/hash and successful
job identity. That preview uses the current Canvas2D/Wasm baseline, not CanvasKit.

The 14 DX acceptance IDs are registered in `quality/features/workbench.json`. The scoped
report at `0aa3e00` has 23 platform/level rows: 21 passed, native Zed interaction and
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
passing happy-path checks above. The repair (`8b22ca7` then `0c3c6a3`, integrated through
`0aa3e00`) registers each helper-created process group before payload launch. Cancellation
blocks descendant admission, signals every registered group and requires confirmed group
disappearance before releasing ownership. A nested timeout preserves sibling/unrelated jobs.
Payload exit status remains separate from supervisor termination. External self-daemonizing
tools outside this protocol and Windows tree cleanup remain unsupported.

The worker's 44-case cohort passed. Fresh independent review found no actionable issue;
27 focused test results passed, including nested cancellation, resistant descendants,
supervisor death and permission-error handling. Initial reviewer fixture-write sandbox
failures were retained and affected checks subsequently passed in writable locations.
Earlier repair attempts, including an invalid externally released test, remain under
`artifacts/evidence/025-owned-process`. Darwin sometimes returned transient `EPERM` during
group teardown; the repair retains those diagnostics and only succeeds after every group
returns `ESRCH`. Persistent denial and other cleanup failures remain fatal.

The first integrated browser attempt at `0aa3e00` passed 13/14 journeys. A concurrent
full-check negative gate temporarily created deliberately invalid untracked source files;
the dashboard retained a dirty snapshot, then refused initialization when source changed.
Preview navigation consequently timed out. That packet remains at
`025-workbench-0aa3e00`; it is not final acceptance. The separate serial packet passed all
14 journeys with clean source, including the actual exported snapshot. Exact-source browser
evidence must run separately from negative gates that intentionally mutate the checkout.

## Remaining acceptance and scope

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
