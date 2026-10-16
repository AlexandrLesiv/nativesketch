# Rust ownership

Keep platform-independent behavior in sketch-core and native interop in sketch-ffi.
Read docs/architecture.md when changing an interface or module boundary. Preserve
editable document semantics independently of renderer caches. Document FFI ownership
and never allow unwinding across it. Run cargo fmt, applicable tests, and clippy.
The existing editor has transactional drawing and persistence APIs. Preserve their behavior
while extracting modules; change behavior only in a separate specified step. Read
docs/architecture/invariants.md and docs/architecture/contracts.md for the target boundaries.
One owner changes shared contracts. Preserve ABI 2, Wasm protocol 1 and document bytes
during structural extraction; version intentional changes with consumer conformance tests.
Use at most 200 physical lines per new/extracted source file, including tests. Keep unsafe
code isolated to bridges; no document dependency on native UI, Skia or filesystem policy.
Add invariant/property and malformed-input tests where relevant; replay is not native E2E.
Provide structured error context without user drawing content. Validate entire batches
before mutation and acknowledge saves by document session plus revision.

