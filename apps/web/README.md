# Browser host

Run `./scripts/run-web`, then open http://127.0.0.1:8077. Requires Rust with the
`wasm32-unknown-unknown` target, Node/npm and Python 3. `./scripts/check-web` compiles
pinned TypeScript and tests this adapter against the actual Rust/Wasm binary.

Semantic HTML controls surround a provisional Canvas2D painter of Rust path commands.
All document, geometry, selection, history and persistence semantics remain in Rust.
The local service worker caches versioned application assets after the first successful
load; runtime assets have no external network dependencies. Reload after updating the
build to allow the new worker to activate when old tabs close.

Pen draws with mouse, pen or touch; Select hit-tests and exposes corner handles.
Use numeric scale or Previous/Next stroke for keyboard-accessible selection. Two
contacts cancel draft ink/resize and navigate; remaining contacts never resume ink.
Space-drag and scrolling pan; Control/Command-scroll zooms. Escape/focus loss/capture
loss/layout changes cancel temporary input. Text controls retain their editing keys.

Open validates a bounded .nsketch file before replacing the document. Download copy
exports bytes but deliberately **does not clear Unsaved**: browser download initiation
cannot prove the file was written. New/Open offer Cancel, Download, or explicit Discard.
There is no automatic recovery of unsaved drawings after a tab/process closes.

Canvas2D is disposable presentation, not a production renderer decision. Physical pen
feel and browser accessibility speech need device testing; Wasm adapter checks alone
are not UI evidence.

## GitHub Pages

The public browser app is deployed to https://alexandrlesiv.github.io/nativesketch/.
This is the current interaction prototype: Rust/Wasm owns editing, Canvas2D paints it;
renderer task 002 remains open. Serving the app does not upload drawings or add cloud
storage. Unsaved drawings still need an explicit download before closing the tab.

`.github/workflows/pages.yml` pins the build toolchain and official Actions. It runs
core, Wasm conformance and browser adapter checks on Ubuntu before publishing only
`artifacts/web/`. All asset URLs and the service-worker scope are relative, supporting
the `/nativesketch/` project path as well as a local root URL.

Repository setup: Settings → Pages → Source → GitHub Actions. The `github-pages`
environment must allow `main` and `codex/github-pages`. The latter is the explicitly
authorized initial publication branch so the demo can be served before source PRs are
merged. Pull requests build/test only and cannot deploy. After merging the Pages PR,
remove the bootstrap branch and its deployment allowance; future updates come from
`main`. Manual dispatch is also restricted to those same branches by the job conditions.
No workflow merges branches or publishes native installers.

The worker caches the app for offline reload after a successful first visit. Close all
app tabs and reopen to activate an updated worker. This is asset caching, not drawing
recovery. GitHub Pages supplies HTTPS and Wasm MIME delivery; verify these on the live
URL after deployment. Build failures leave the previous deployment in place.
