# Shared viewport preparation evidence

Worker `7245374f8e516034b21016d6bc3421ea98c3187f` integrates as `e40bd6f`.
The [frozen contract](../architecture/020-shared-viewport-preparation.md) moves pure f64
clipping/preparation from the native Skia adapter into `sketch-core`. Native Skia retains
only renderer-specific path conversion. Document semantics and bytes do not change.

The additive portable queries prepare a selected immutable scene stroke for actual backing
dimensions, page its complete physical-pixel commands and release the independent result.
Existing scene queries remain unchanged. One prepared lease may exist per application;
its result survives source-scene release, but keeps application shutdown busy until release.
Output capacity is reserved before acquisition. Preparation has explicit aggregate vector
capacity, emitted-point, work and subdivision-depth bounds; failure publishes no partial path.

At the exact clean worker commit:

- 192 affected Rust tests passed: 128 core, 39 wire, 13 bridge and 12 native CPU renderer.
  New cases cover actual fractional/asymmetric backing ratios, transformed signed endpoints,
  multiple contours, local-path reuse, budget failures, exact wire bytes and lease lifetime.
  Existing giant enclosing/grazing curve and single-alpha assertions were retained.
- Actual C and release Wasm consumer conformance passed with clean source identities and
  valid environments. The tests exercise multipage actual outlines, insufficient acquisition
  output, source release, shutdown with held preparation and exact release semantics.
- Strict Clippy, Rust formatting, relevant Oxfmt/type-aware Oxlint, narrow Wasm consumer
  types and source/dependency gates passed. No source file exceeds 200 physical lines.

Fresh independent review found no actionable issue. It independently reran 12 focused tests,
the actual C consumer and release Wasm conformance, verified binary hashes, and inspected
allocation/paging/ownership. It reused the recorded Rust build binaries rather than rebuilding
the full suite. No GUI, GPU, presentation or physical-latency check ran.

Raw packets are retained in `artifacts/evidence/020-shared-viewport-worker/`, including the
exact conformance packet `portable-exports/1792129330993-bba44f34-af53-46a4-9ad7-1450fed4cdeb`.
Native C binary SHA-256: `c329384e7b8f46441867656feaca9bfd948580741139a986a329fb671482142a`.
Wasm SHA-256: `c577f71425c109eed816477e636a48ec81344b9aa07ccefd39e3ef64a75d7a55`.

Two earlier C attempts failed because the fixture detached a dirty document without the
existing navigation guard. The repair executes the actual Close → Discard → ConfirmClose
workflow before isolating prepared-lease shutdown. Both failures remain retained. Full root
TypeScript checking on the older worker base lacked the later Playwright dependencies;
that is not reported as a passing check. Combined primary verification remains required.

At clean primary `760a286`, the full background integration passed, including the combined
TypeScript dependencies, 140 tooling tests, Rust workspace and actual C/Wasm consumers,
macOS build/pre-AppKit smoke and 17 GPU-free Swift tests. The scoped
[viewport report](../../artifacts/evidence/020-viewport-760a286-retry1/coverage.html) reran
12 mapped Rust tests and imports the same-source C/Wasm packet: all seven acceptance rows
passed with no mapping issues. Its initial report assembly failed on an optional metadata
field, before publication; that attempt is retained separately and not treated as acceptance.

## Browser consumption

Frozen worker `3e02a7045e2b98a5917fd33e277e3b9e6a1bac08` integrates as `6e0c4e1`.
The browser now pages and releases actual prepared-path leases, converts their physical
coordinates without a second transform, and keys disposable paths by geometry, transform,
leased viewport and actual backing dimensions. Ordinary local geometry remains reusable.
Complete staging precedes canvas clearing. Rejected pages or capacity leave previous pixels
and paths intact; rejected release retains ownership, while failure after an actually
consumed release quarantines the instance rather than repeating that command.

The temporary 4,096-device-coordinate rejection guard is removed. Actual huge spanning
strokes now render through shared Rust preparation. Existing explicit work/memory/point
bounds still reject excessive preparation; this is not unbounded geometry support.

At the exact clean worker revision, all 29 classified Node checks and 17 headless browser
journeys passed, along with strict TypeScript, full type-aware Oxc and formatting. Added
checks cover signed giant crossings against local references, fractional/asymmetric backing,
enclosing/grazing interiors, empty prepared output, whole-path alpha, cache identity,
malformed/rejected pages, release retry, capacity and consumed-release quarantine. The
browser imports an actual giant portable document, checks its rendered coverage and downloads
unchanged bytes. No testing-only production controls were added.

Fresh independent review found no actionable issue and independently passed 12 focused
actual Wasm/CanvasKit and injected failure checks. It inspected the clean frozen 29/17
packets without duplicating browser execution. The lead inspected the actual giant-stroke
capture. Worker packets/captures are retained under `artifacts/evidence/020-web-prepared-worker/`.
All changed source files meet the 200-line cap. Combined primary verification follows.

Preparation is bounded synchronous render work, not a latency optimization claim. Native
GPU acceptance, additional browser/device combinations and task 024 remain open.
