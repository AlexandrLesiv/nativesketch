# Task 017 feature evidence foundation

Implemented from `4c8355f` in isolated evidence/native worktrees, integrated as `1d34241`
and `fcebf8d`. The native test file's five registrations subsequently gained explicit
`await` for the shared type-aware lint gate. This records local preparation, not task
acceptance: actual macOS UI execution and prerequisite owner acceptance remain pending.

## Implemented and verified

Cargo test binaries and Node structured runner events supply actual executable IDs.
Checked-in schemas, runtime validation and exact source/binary/platform matching feed
the HTML/JSON report. It preserves missing, failed, skipped, blocked, stale and flaky
evidence; report generation is distinct from a passing coverage gate. Source fingerprints
include nonignored untracked bytes. Logs and per-run history remain ignored artifacts.
Commands and retry/import semantics are in [the runner guide](../quality/evidence-runner.md).

At `fcebf8d` plus the five await fixes, `node scripts/check.mjs` passed: 40 Node tooling
tests, 32 Rust tests, rustfmt/strict Clippy, Oxfmt/type-aware Oxlint/TypeScript, source-size
and dependency gates, Swift application build and Swift/Rust ABI smoke verification.
The smoke is injected integration evidence, not native pointer E2E. The source-size
gate still reports twelve oversized legacy files assigned to extraction tasks.

Fresh reviewer independently checked the same runtime changes, ran all 14 new evidence/
discovery/XCTest-normalization tests and verified deterministic same-source signing for
native retry identity. No actionable code findings remained. Review repaired false-pass
risks from failed Node suite cleanup and XCTest finalization, missing source rechecks,
and retry identity. Negative fixtures now protect those cases. Reviewer did not run UI.

## Native and platform limits

The macOS XCTest target and actual app build. Its intended journey traverses native
brush controls, pointer input, visible red ink, saved geometry, normal quit, relaunch,
Open and Save As. Disposable signed app identity isolates it from an ordinary running
Native Sketch instance. The [native evidence record](017-macos-e2e-foundation.md) describes
the independent document/pixel assertions and setup failure.

The execution account differs from the active console user; XCTest cannot authenticate
off console. Enumeration returned setup errors even with exit zero and is rejected.
There are no discovered/passing native cases yet; runtime selectors remain unverified.
The user-session action is pending. Other platform UI drivers, physical pen-to-screen
latency and final source-cap acceptance have not been completed by this milestone.

Task 018 may prepare compatibility-preserving extraction under the authorized sequence,
but cannot treat blocked UI acceptance or incomplete task 016 host parity as passed.
No main merge, release, global permission change or publication was performed.

## Integrated oracle and report verification

Designer review required independent evidence in every sampled pixel patch and absolute
document endpoints. The repaired journey also requires identity stroke transform, closing
a reviewer finding that raw points alone could hide translated/scaled saved geometry.
Pure Swift regression checks run automatically before XCTest preparation. Fresh reviewer
rechecked `c11ed4eb6c5fd21baa64730df27e3b3d9bcb213d` and found no remaining actionable
code findings; this was not runtime UI approval.

At that clean integrated revision, `node scripts/test-e2e.mjs --platform macos
--prepare-only` built the app and XCTest target and passed all ten pure oracle checks.
It intentionally exited 1 after preparation with empty native discovery/results and
an explicit deferred-execution record. Evidence is under ignored
`artifacts/evidence/macos-1792066288510/` on the integration checkout:

- Isolated app SHA-256: `488fdcb70ce3c5d3cb21384a87d36d730dfbcec191336b14220a1c72fe0054d6`.
- XCTest binary SHA-256: `156b948cd0d257e5603491e14bf34d2dc1ed5710e95bac9a75065731a3a3284d`.

Actual Cargo/Node evidence plus those empty native envelopes generated 147 requirements:
five passed compatibility cases, twelve explicitly incomplete cases and 130 missing rows,
with no mapping errors. Scoped COMPAT-001 through COMPAT-005 gating passed. Overall
coverage remains incomplete. `artifacts/evidence/coverage.html` links execution evidence;
history retains this exact source identity when later runs replace the current report.
