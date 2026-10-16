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

Preparation is bounded synchronous render work, not a latency optimization claim. Browser
consumption, cache keys and real CanvasKit large-stroke rendering remain the next separate
verification slice. Native GPU acceptance and task 024 remain open.
