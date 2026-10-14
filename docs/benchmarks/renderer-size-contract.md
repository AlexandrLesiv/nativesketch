# Separate renderer bundle-size investigation

Scope: macOS arm64 on the existing benchmark host. This is a packaging experiment,
not another performance run or a production backend commitment.

Build four variants from one clean source revision and lockfile: common blank baseline,
Skia only, Vello GPU only, and both. Use the same Swift -O, Rust release/thin-LTO,
macOS deployment target, host, fixtures and bundle layout. Default benchmark behavior
remains both engines. Unavailable backend requests must fail; the baseline must be
explicitly identified as a blank target and excluded from rendering comparisons.

All variants retain the experiment's common wgpu/Metal infrastructure and vello_common
geometry utilities. Skia only excludes the Vello GPU renderer, but is not a minimal
Skia-to-Metal integration without wgpu. Vello only excludes Skia. Keep the existing
Skia JPEG/PDF-enabled binary feature package; unexercised APIs and dependency features
are not assumed to survive dead stripping. Do not estimate their shipped cost from
static archive sizes.

Record two equally treated lanes: the linked release bundle, and a copy stripped with
Apple strip -S -x then ad-hoc signed. Preserve originals. For each lane record logical
bundle bytes, executable bytes, exact file hashes, deterministic ZIP/deflate-9 bytes,
and deterministic executable gzip-9 bytes. Neither compressed value is an App Store
or notarized download estimate. Exclude analysis logs and link maps from app bundles.

Inspect architecture, system dynamic-library dependencies, Mach-O sections, selected
Cargo dependency graph and linker maps. Compare additions over the common baseline
as linked adapter/renderer increments within this harness, not universal library size.
Also show combined-minus-single marginal costs to expose shared-code effects.

Run short native correctness smoke trials for both single-renderer variants and the
combined build; confirm images and actual backend selection, not just executable launch.
Run the blank baseline and disabled-backend failure cases separately. These runs are
not performance evidence. Preserve earlier measured executables and accepted datasets.

Deliver exact build/provenance, machine-readable size tables and an HTML report update.
Obtain fresh independent review; run affected checks. Leave renderer selection open and
main unmerged. iOS/iPadOS, Android and browser sizes are not inferred from macOS results.

## Additional linker optimization lane

The original maps showed no explicit unused-code removal. Preserve that audit and
re-link the exact cached per-variant Rust archives with the identical Swift source and
`-dead_strip`. Verify archive identity and embedded metadata before linking; Rust is
not rebuilt. Preserve original build.json and record the additional flag, complete
invocation and archive/input/output hashes in separate relink provenance. Audit this
lane independently, before and after symbol stripping; never silently replace defaults.

Both lanes receive the same 38 short native correctness runs (eight fixtures through
each single/combined stripped renderer, four linked live cases, two blank controls),
plus three unsupported-backend rejections. Require exact decoded output equality for
the same backend/fixture across build, strip and linker lanes. This is static fixture
coverage, not all possible production rendering behavior or performance evidence.

## Reproduction

Build with `scripts/build-renderer-comparison release <variant>` for baseline, skia,
vello and both from one clean revision. Use `scripts/audit-renderer-size --help` for
the matched packaging audit and `scripts/check-renderer-size-smoke --help` for native
correctness checks (the latter needs Pillow). The additional relink lane is produced
by `experiments/renderer-comparison/build-size-dead-strip --help`; retained Cargo
archives must match the original maps and embedded metadata. Generate the report with
`scripts/build-renderer-size-report` after exporting audited JSON.
