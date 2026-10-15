# Task 021A: typed save-contract evidence

Implementation source: `8fe29d62af93ef3ce6d1e2350b47c6e2a71e1c62`, isolated branch
`codex/storage-recovery-services`. Benchmark validation follow-up: `272bf24` (only the
example changed). This is portable service preparation, not native asynchronous-save or
recovery acceptance. Main and existing ABI/Wasm consumers are unchanged.

Host: Mac14,10, macOS 27.0.1 (26A434), Rust 1.99.0, local Node 24.22.0.
Repository dependencies installed with `npm ci --ignore-scripts --offline`.

## Verification

At clean `8fe29d6`, `node scripts/check.ts --core` passed: 44 Node tests, 76 Rust tests,
strict TypeScript/Oxlint/Oxfmt, Rust format/Clippy, source-size and dependency gates.
Six existing oversized host files remain migration debt. No new source exceeds 200 lines.
`node scripts/check.ts --platform wasm` passed actual Node/Wasm/native byte conformance
and all 20 frozen protocol-v1 request/response pairs. Existing ABI2 and document fixtures
passed in the Rust suite. No native app or foreground UI automation ran for this phase.

At clean `272bf24`, Rust format and strict all-target Clippy passed after the example-only
review correction. The release snapshot example consumed every captured snapshot and
asserted its encoded bytes equal the original document outside the timed interval.

Raw logs, actual discovered test names, benchmark JSONL and SHA-256 summary are retained
under ignored `artifacts/verification/021a/` in the implementation worktree. Discovered
names are input to the lead-owned acceptance registry; this document does not claim all
task021 host/recovery requirements have executed.

The 25 added tests cover request-time immutable copies, FIFO single-active ownership,
slot/payload limits, encoded-size overflow, edit/export permissions, idle-only admission,
cancelled predictions, unknown/duplicate completions, saved-revision undo/redo ABA,
session replacement, completion during a new draft, Save As destination adoption in both
acknowledgement orders, pending destination protection, same-path stale writes, uncertain
outcomes, cancellation before/after writes, and failure retention. Relevant IDs:
SAVE-001/002/003, VIEW-001, INK-002 and ASYNC-001. Existing LOAD-001 decoder tests remain.

## Synchronous snapshot cost

Command: `cargo run --release -p sketch-core --example snapshot_cost` at `272bf24`.
One warm-up plus 20 measured iterations per synthetic document; compiler barrier after
capture, encoding/byte verification and destruction outside the measured interval.

| Samples | Encoded bytes | Charged snapshot bytes | Median copy µs | Maximum copy µs |
| --- | ---: | ---: | ---: | ---: |
| 1,000 | 48,120 | 48,136 | 2.292 | 5.958 |
| 10,000 | 480,120 | 480,136 | 13.959 | 20.250 |
| 100,000 | 4,800,120 | 4,800,136 | 125.625 | 481.208 |
| 1,000,000 | 48,000,984 | 48,001,144 | 943.771 | 1,306.709 |

Release example binary SHA-256:
`d5cb36ca22051a7bf3e481698645d6d85ea396bd1ee92d0bd7227f4fe85c2865`.
These are warm synthetic allocation/copy observations on this host, not latency bounds,
native-event measurements, performance comparisons, or pen-to-screen claims. Earlier
preliminary copy measurements without the compiler barrier/consumption check are superseded.

## Remaining work

Snapshot copying remains synchronous O(document), may delay subsequent event delivery,
and is prohibited in pen callbacks. Encoded worker buffers and allocator overhead require
additional native-service memory budgets. The portable coordinator does not create a
worker, perform I/O, coalesce checkpoints, rotate diagnostics, or implement recovery UI.
Task021B must supervise every worker job and establish staged-store, logging, recovery,
corruption and process-kill evidence; task019 must adopt native asynchronous file actions
and test their actual focus, Save As keyboard shortcut and lifecycle behavior.
