# Browser Skia migration evidence

Production CanvasKit worker `5a03f25` integrates as `3a45709`. Evidence and test follow-ups
through worker `d4b5b68` integrate through `333f9ff`. This replaces Canvas2D painting with
the standard full CanvasKit 0.42.0 bundle while retaining semantic HTML controls, the
portable Rust command dispatcher, issued file work and leased scenes. No new tool is added.

## Ownership and behavior

Rust application Wasm and CanvasKit Wasm are separate instances. Only copied scene data
crosses between them. Complete winding paths are staged before painting, with a 32 MiB
estimated path budget and 4,096-path limit; these are admission bounds, not measured RSS.
The renderer owns and explicitly disposes paths, surfaces, contexts and raster allocations.

Initial WebGL refusal uses explicit Skia CPU raster on the same canvas. A canvas already
bound to WebGL must restore that context or display a rendering error. Context loss cancels
the current interaction and retires local capture/contact state. A failed renderer leaves
the healthy Rust document available for download. It does not report a blank frame as success.

The initial implementation rejects coordinates outside its supported device-space precision
bound before replacing previous pixels. This can affect a large spanning stroke, high zoom
or backing scale, not only distant origins. The shared Rust viewport-preparation follow-up
is required for general spanning-geometry rendering; successful small far-origin tests do
not close that gap. Encoding and preparation still run synchronously outside the pointer
callback. A dedicated engine worker and performance measurements remain separate work.
The later [shared preparation slice](020-shared-viewport.md#browser-consumption) replaces
this temporary rejection with actual large-stroke rendering and separately recorded tests.

## Frozen verification

At clean worker `d4b5b682ec4be3c917e5ab4c61d2bdcfd0e81344`:

- All 16 actual production browser journeys passed in headless Chromium with explicitly
  identified SwiftShader, plus actual CPU fallback after injected WebGL refusal. These cover
  retained drawing/history/files, pixel placement, offline reload, rejected asset update,
  original-canvas fallback, fresh input/undo after context restoration, and exact downloadable
  bytes after injected restoration failure. Synthetic context failure is not physical GPU
  failure; browser chooser cancellation remains an injected event.
- All 20 direct checks passed: 14 portable bridge, five actual CanvasKit CPU raster and one
  TypeScript positive/negative compiler fixture. Their evidence classifications distinguish
  compiler validation from Rust/Wasm execution. The huge-span case verifies safe rejection,
  not successful rendering of that shape.
- Strict TypeScript, Oxc and two evidence identity/classification tests passed. Runtime
  identity hashes both Wasm modules, installed CanvasKit package metadata and declarations,
  scoped browser type compatibility, actual resolved full-module routing and emitted assets.
  A same-version package-export mutation is rejected by a regression test.

Packets, captures and failed attempts are retained under
`artifacts/evidence/020-web-canvaskit-worker/`. The final packets are
`web-canvaskit-d4b5b68-browser` and `web-canvaskit-d4b5b68-node`; each records its own source,
binary, tool and test identity. These are worker-revision results, not a claim that the
later integrated primary revision has independently executed all the same checks.

Initial attempts exposed an acceptance-ID parser limitation, a pixel fixture counting the
empty-canvas hint, a lost-context fixture re-querying an unavailable extension, and test-only
lint/type errors. All attempts remain retained. The parser and fixtures were repaired;
neither production appearance nor pixel acceptance tolerances were weakened.

Fresh independent review cleared production `5a03f25`. Review of the evidence follow-up
found a package-export provenance gap; the final `d4b5b68` review verified its repair and
independently passed the focused tests and type checks. It did not duplicate the browser run.
Designer review inspected the actual raster, restored-WebGL and failure screenshots and
found no interface blocker. The two requested stronger canvas/input assertions are included
in the final passing browser run. Dark/small layouts, speech accessibility, Safari/mobile,
hardware GPU presentation and physical pen-to-screen latency remain pending.

## Integrated primary verification

At clean primary `9124ec5`, the full background integration gate passed: 140 tooling tests,
Rust workspace checks, real native interface consumers, strict Swift lint, the macOS
build/pre-AppKit smoke path and all 17 GPU-free Swift tests. The final source/dependency
gate passed with zero oversized handwritten source files.

The separately executed qualified Node packet passed all 20 checks and the production
browser packet passed all 16 journeys. Their [scoped report](../../artifacts/evidence/020-canvaskit-9124ec5/coverage.html)
has 27 passing acceptance rows and no mapping issues. Referenced artifacts are copied into
that report's directory, so its links remain local and complete. Exact-source primary
packets are `020-canvaskit-node-9124ec5` and `020-canvaskit-browser-9124ec5`.

The workbench subsequently passed 17 service and 14 browser checks, including its actual
CanvasKit app preview, in `025-workbench-canvaskit-9124ec5`. No desktop app was opened.
Native Zed and speech accessibility acceptance remain separate and pending.

Two earlier primary integration attempts failed on test-fixture type-aware lint and pixel
helper formatting. The lead diagnosed and repaired these, ran the complete tooling gate,
and included the new renderer-evidence regressions in its normal inventory. The failed logs
`integration-canvaskit-05952cf.log` and `integration-canvaskit-retry1.log` remain beside the
passing `integration-canvaskit-retry2.log`. Fresh review cleared the explicit-receiver fix.

This packet does not close task 024 or authorize a merge or release.
