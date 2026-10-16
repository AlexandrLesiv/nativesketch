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

Canvas2D is interim presentation; Skia/CanvasKit is selected but not yet integrated here. Physical pen
feel and browser accessibility speech need device testing; Wasm adapter checks alone
are not UI evidence.

## GitHub Pages

The public browser app is deployed to https://alexandrlesiv.github.io/nativesketch/.
This is the current interaction prototype: Rust/Wasm owns editing, Canvas2D paints it;
the selected Skia/CanvasKit consumer migration remains pending. Serving the app does not upload drawings or add cloud
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

## Adapter composition and quality boundary

`main.ts` only assembles the host. `host.ts` owns browser presentation/capture state and
binds callbacks; `presentation.ts` handles availability, status and failure UI;
`canvas-painter.ts` retains Canvas2D rendering. `controls.ts` and `keyboard.ts` retain
semantic control and native text-field behavior. Pointer down/move/finish and coalesced
sampling have separate input modules; `viewport.ts` owns wheel/blur/resize wiring.
`files.ts` preserves asynchronous replacement guards and download-does-not-acknowledge-save.
`startup.ts` loads the existing Wasm protocol 1 engine and worker. `engine.ts` retains
its existing ownership, 64-bit identity and copy-before-next-command behavior.

This is mechanical decomposition. Rust portable protocol v2, CanvasKit, a Worker-owned
engine and the full shared application-command migration remain separate task 022 slices.
No control, layout, file semantics or service-worker activation policy was added here.

The supported Node build (`node scripts/build.ts --platform web`) uses root TypeScript 7
strict/erasable types and Vite. The retained nested TypeScript/Prettier setup remains for
legacy parity work. Root Oxfmt formats all new sources; files stay <=200 physical lines.
The explicit `.ts` imports are rewritten to `.js` by the nested compiler as well.

Vite emits `module-inventory.json` with every output and transitive import. The build rejects
missing, unexpected or unsafe output paths and unknown imports. Every emitted payload,
including the inventory, contributes to the service-worker version and precache. The worker
and `.nojekyll` are generated deployment metadata, not exclusions for application modules.
A new helper module therefore cannot silently work online while failing on offline reload.

`checkWeb` runs the actual Rust/Wasm adapter probe and four production asset checks: all
modules/subpath URLs precached, transitive-content cache invalidation, incomplete graph
rejection, and failed cache installation preserving the previous cache. `checkWasm` adds
real native-byte conformance and the 20-pair protocol-v1 fixture. These support INPUT-001,
CANCEL-001, UNDO-001, LOAD-001, SAVE-001 and FFI-001; Node worker simulation is not real-browser
UI/offline acceptance. A separate browser lane must still verify focus, drawing, controls,
service-worker lifecycle, offline reload and downloads using the emitted build.
