# Native runtime and C bridge verification

Code revision: `c82f027254ace9c95c8dd29b6b1d8c35d6b68263`.
Host: macOS27.0.1(26A434), arm64; Rust1.99.0; Node24.22.0.
This records background Rust/C/Wasm evidence; it does not grant native UI acceptance.

## Executed checks

- `node scripts/check.ts --core`: passed format/lint/type/source/dependency/Node and Rust gates.
- `cargo test --workspace --all-features`:145 passed, zero failed, two ignored subprocess
  entrypoints. Both entrypoints were invoked by their passing parent process tests.
- Strict workspace Clippy (all targets/features) and rustfmt: passed.
- `node scripts/check.ts --platform wasm`: real Wasm behavior and20 fixed protocol-v1
  request/response pairs passed; ABI2/document fixture tests also passed.
- Real C11 consumer compiled and linked the default release static library, then ran successfully.
  It checks fixed layouts, rejected whole-input-batch state preservation and owned lifecycle.
- Default release dylib excludes the test-control symbol; explicit feature build exports it.
  The enabled-feature immutable SaveA/continue editingB test passed. Default debug was restored.

Logs and executable discovery are under `artifacts/verification/019bc/c82f027-*` in the
storage worktree. The JSON manifest records host/toolchain, totals and the actual C binary SHA256.
C binary: `46e6619f8bb1f33f411b8eea4f489f33ee864392667340bb55653372ea3ea841`.
Raw C execution is macOS bridge evidence, not a portable Node unit test.

## Behavior tests added for runtime ownership

These IDs are taken from actual Cargo executable discovery, not source-text matching:

- `resolution_ready_preserves_new_stroke_and_saves_exact_resolved_admission`
- `poisoned_detach_drains_accepted_write_and_releases_grant_once`
- `explicit_save_gate_keeps_immutable_a_while_owner_commits_b`
- `failed_save_as_returns_correlated_save_as_retry_not_old_binding_save`
- `cancelled_recovery_must_not_follow_later_new_document`
- `replacing_session_clears_checkpoint_revision_knowledge`
- `first_checkpoint_allocation_failure_retries_after_capacity_is_released`
- `rejected_identity_cannot_be_authorized_by_window_or_global_retry`
- `failed_read_and_recovery_export_redacted_correlated_records`
- `independent_root_locks_survive_conflict_and_release_on_process_death`
- `global_resolution_barrier_and_repeated_stale_reads_preserve_new_ink_and_release_leases`
- `exclusive_recovery_root_degrades_second_runtime_without_blocking_local_io`
- `disabled_recovery_is_observable_and_ordered_registration_can_be_listed`
- `local_resolution_accepts_macos_var_alias_but_rejects_symlink_final_entry`
- `idle_wake_exists_only_for_pending_schedulable_checkpoint`
- `boundary_tests::caught_boundary_panic_preserves_dirty_detach_guard_then_allows_terminal_cleanup`

The last test executes the real internal FFI unwind guard, verifies dirty normal detach is
rejected, confirms poisoned calls preserve output, then uses public state/detach/poll/shutdown/
free functions. There is no public production panic trigger.

## Review and remaining limits

Independent review is checking the exact final freeze. Earlier review exposed recovery-origin,
registration retry and diagnostic correlation failures; each now has a targeted regression.
The lead retains the independent review records and native Swift/UI evidence separately.

Only trusted local filesystem adapters are implemented. Provider coordination, sandbox grants,
ACL/xattr preservation, adversarial filesystem races and remote storage remain future adapters.
Snapshot capture remains an explicit O(document) owner operation outside input callbacks;
encoding, filesystem I/O, recovery and retirement run on the serial worker. No new physical
pen latency, GPU performance, power-loss guarantee or unsupported-platform claim is made.
