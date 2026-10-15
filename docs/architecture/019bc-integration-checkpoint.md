# Native runtime integration checkpoint

This is the first linkable 019B/C tranche, not completion or native acceptance. It adds
`sketch-native-ffi` (host-only tests; core + services dependencies), the versioned C DTOs,
one-owner ApplicationSession routing, a bounded-operation serial native worker, local
resolution/read/write, recovery inventory/checkpoint wiring, redacted export, and lifecycle.
Services and native bridge require the pinned Rust1.99 standard file-lock API. ABI2/Wasm1
and document fixtures remain separate and unchanged.

Normal detach waits for owned operations. Explicit poisoned detach retains a runtime tombstone:
accepted I/O finishes, trusted results settle without claiming clean or presenting obsolete UI,
grants release, and the editor retires off-owner. Poll/release/shutdown remain available when
the C boundary is poisoned. Native UI owns the unsaved-loss warning.

Local root container layout is `owner.lock` plus private `sessions/` for recovery. Diagnostics
has its own OS lock. Empty registrations verify identity without retaining a directory; the
first checkpoint allocates storage. Startup/list prune only verified empty directories. No
unresolved checkpoint is removed by empty cleanup. The existing seven-day exact-marker expiry
runs at root opening. This retains the trusted exclusive local-filesystem assumption; there
is no sandbox/provider coordination or adversarial path-race guarantee.

Initial checks on macOS27.0.1 arm64 / Rust1.99 / Node24.22:

- Full `node scripts/check.ts --core` passed, including dependency and source gates.
- Workspace tests, rustfmt and strict all-feature Clippy passed.
- A real C executable linked the static library and passed layout, invalid whole-batch and
  asynchronous close/free checks (`crates/sketch-native-ffi/tests/consumer.c`).
- New runtime tests verify resolution readiness preserves new ink, exact admission writes,
  poisoned accepted-write drain/grant release, recovery lock conflict/degraded local I/O and retry.
- Logs: `artifacts/verification/019bc/initial-{background,workspace-tests,clippy,c-consumer}.log`.

Remaining before a final freeze: deterministic feature-gated controls; broader capacity,
failure/cancellation/recovery/second-process tests; actual Wasm compatibility; independent
review and repairs; linked Swift/native E2E. The current feature name is reserved but does
not yet export the test hook. These gaps are not passes or promises of UI/performance behavior.
