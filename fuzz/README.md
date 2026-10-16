# Bounded core fuzzing

Run `node scripts/check-fuzz.ts --output artifacts/evidence/NEW_RUN` from the repository
with the pinned Node. The output directory must be new. The runner uses preinstalled
project-local cargo-fuzz 0.13.2 and nightly-2026-10-15; it never installs dependencies,
changes the default compiler or opens an application. This first lane is verified on
macOS ARM64 only. Provision tools explicitly in `.build-cache/toolchains` before running.
The standalone lockfile pins libfuzzer-sys 0.4.13 and its dependencies. The upstream
fuzzer contains C++; application semantics and all owned fuzz targets remain Rust.

Targets:

- `document_codec`: raw bytes plus independently checksum-repaired candidates exercise
  framing and deeper validation. Failed imports preserve existing bytes and revision;
  accepted imports round-trip to identical documents/canonical bytes and finite paths.
- `command_sequence`: at most 128 typed editing/input operations, including stale contexts,
  predictions/corrections, invalid values and transaction conflicts. Rejection must preserve
  the projection, committed bytes and live geometry; every committed snapshot remains valid.

The asserted properties cover malformed-import preservation (LOAD-001), rejected-batch
atomicity (INPUT-001), canonical snapshots, finite geometry and stale typed-command
rejection. Predictions, cancellation and pressure are generated stimuli; this harness does
not independently prove prediction exclusion, undo cardinality, pressure/order semantics,
native cancellation (CANCEL-001) or asynchronous callback replacement (ASYNC-001).
These are bounded coverage-guided smoke checks, not exhaustive or native/physical tests.
Fixed seed 2601016, at most 25,000 runs/30 seconds per target, five seconds per input,
1 GiB RSS and 64 KiB document/1 KiB command inputs bound each execution. Initial document
and command corpus files are copied into the evidence directory; discoveries and crashes
remain there. Investigate and retain a crashing input before adding a minimized regression.
Do not delete crashes, lower limits or narrow an oracle merely to obtain a passing run.

The runner records exact source, tool and target hashes, compiler flags, bounded logs,
exit/failure records and complete libFuzzer summary. It verifies core sanitizer/fuzz flags
and actual binary sanitizer symbols; source mutation, cancellation or incomplete output
cannot publish a pass. The separate ASan lane retains its stronger deliberate-fault canary.
No GUI, Skia, JNI, worker/FFI unsafe code, filesystem durability or other-host acceptance
is inferred from these safe-core targets. Add those as separate scoped targets when needed.

Normal integration checks format these sources. The explicit lane builds instrumented
targets; `cargo clippy --locked --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings`
checks the standalone harness. The Node proof-validation tests run in the normal tooling gate.

Sources: [Rust Fuzz guide](https://rust-fuzz.github.io/book/cargo-fuzz/guide.html),
[bounded fuzz smoke](https://rust-fuzz.github.io/book/cargo-fuzz/ci.html),
[libfuzzer-sys](https://docs.rs/crate/libfuzzer-sys/0.4.13).
