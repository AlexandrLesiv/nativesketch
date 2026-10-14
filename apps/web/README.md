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
