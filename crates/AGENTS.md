# Rust ownership

Keep platform-independent behavior in sketch-core and native interop in sketch-ffi.
Read docs/architecture.md when changing an interface or module boundary. Preserve
editable document semantics independently of renderer caches. Document FFI ownership
and never allow unwinding across it. Run cargo fmt, applicable tests, and clippy.
There are no drawing APIs in the bootstrap; design the task contract before adding them.

