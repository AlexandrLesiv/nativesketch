# Project workbench interface

Seven semantic HTML views share one browser bundle and styles between the loopback service
and self-contained offline exports. Foreground actions remain disabled. This interface does
not change task acceptance or turn stale execution evidence into a current-source pass.

Build from repository root with the pinned Node runtime:

```text
node node_modules/vite/bin/vite.js build --config apps/workbench/vite.config.ts
node node_modules/typescript/bin/tsc --project apps/workbench/tsconfig.json
node node_modules/typescript/bin/tsc --project apps/workbench/e2e/tsconfig.json
node node_modules/oxlint/bin/oxlint --type-aware apps/workbench
node node_modules/oxfmt/bin/oxfmt --check apps/workbench
node node_modules/@playwright/test/cli.js test --config apps/workbench/e2e/playwright.config.ts
```

The runner is headless only. Set `SKETCH_BROWSER_EXECUTABLE` to the provisioned Chromium
headless shell when it is outside Playwright's default cache. Test discovery uses the same
command with `--list`. No browser, IDE, native app or simulator is opened automatically.

Build output: `dist/index.html`, `dist/assets/workbench.js` (one IIFE), and
`dist/assets/workbench.css`. The service supplies `script#workbench-bootstrap` containing
frozen Bootstrap JSON; `#workbench-root` is the mount. Static mode never fetches a session,
polls jobs, refreshes or executes actions. Service capabilities stay in memory and headers.

Browser tests use the real built bundle. `offline.spec.ts`, `invalid.spec.ts` and
`large-document.spec.ts` use explicit synthetic snapshots in disposable files;
`live.spec.ts` uses a disposable HTTP protocol fixture and never starts a child command.
These verify interface behavior, not service process cleanup or repository adapter correctness.
The root integration lane verifies the actual service and export producer separately.
Retain each runner JSON, failure trace and screenshot before rerunning; output is ignored.

Acceptance coverage includes DX-NAV-001, DX-SOURCE-001, DX-TASK-001, DX-DOC-001,
DX-COVERAGE-001, DX-EXPORT-001, DX-ACCESS-001 and the interface portion of DX-JOB-001.
Dependency and metric tables preserve provenance/missing states; security, real action process
ownership, actual Wasm preview and Zed launch require their separate integration evidence.
Native host acceptance, screen-reader speech and physical pen latency are not established by
these headless browser tests. Screenshot fixtures visibly identify their fixture revision.
