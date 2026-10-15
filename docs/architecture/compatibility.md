# Task 015 compatibility baseline

Captured from `e467d402c0f9888ae11a5312a2caabcd9335480f`, whose runtime source is unchanged
from `codex/github-pages@712059d5c4470c9770d08c42de8f1391884a66be`. This is a preservation
baseline, not certification of platform E2E, rendering correctness or future contracts.
Never regenerate fixtures as part of a passing test. A deliberate format/behavior change
requires an explained version/migration decision and independent review.

## Fixed fixtures and executable coverage

| Fixture / executable test | Protects | Does not prove |
| --- | --- | --- |
| `crates/sketch-core/tests/fixtures/portable-v1.nsketch` | Fixed NSKETCH1 bytes: pen pointer 42, brush(.2,.3,.4,.5)/6, actual(10,20,t1,p.5) and(30,40,t2,p.7); predicted(80,90,t3,p.9) omitted | Every valid document or format migration |
| `document_compatibility::committed_predictions_match_fixed_document_bytes` | INK-002, UNDO-001/002; independently recreate stroke and compare fixed bytes; active export/undo reject | Native pen delivery |
| `document_compatibility::frozen_document_roundtrips_and_invalid_import_retains_history` | LOAD-001; fixed decode/encode, every truncated prefix rejected without history loss | All malformed 64 MiB inputs |
| `document_compatibility::invalid_batch_and_cancel_preserve_frozen_document` | INPUT-001, INK-001, UNDO-002; invalid trailing sample cannot change preview; cancel preserves bytes/history | Every prediction/correction permutation |
| `document_compatibility::correction_requires_an_existing_id_and_identical_timestamp` | INPUT-002; correction identity and timestamp preservation | Device clock conversion |
| `abi_compatibility::abi2_native_layout_is_unchanged` | ABI 2 struct sizes/important offsets on supported 64-bit targets | C compiler layout on an unexecuted target or every header symbol |
| `abi_compatibility::abi2_frozen_document_capacity_and_save_revision_contract` | FFI-001 output ownership/capacity, SAVE-001/002 stale revisions, undo revision identity, fixed bytes | Thread safety misuse, panic injection, physical disk durability |
| `protocol-compatibility.ts` + `fixtures/protocol-v1.json` | Actual Wasm1 fixed request/status/response bytes: state, begin, append, commit, render, snapshot, undo/redo, malformed requests, corrupt import, new, stale acknowledgement | Independent correctness of captured geometry, browser/native E2E |

Rust integration test names are prefixed above with their integration binary names; Cargo
lists each function within that binary. JSON stores literal input/output hex and capture
revision. The nonempty binary document was captured by the unchanged native
`portable-fixture` example; the Wasm trace was captured independently from the built raw
Wasm instance using the same synthetic stroke. Runners compare checked-in constants;
there is no checked-in golden-regeneration command. Existing conformance also compares a
fresh native export against Wasm, complementing (not replacing) the fixed baseline.

Run:

```text
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
./scripts/check-wasm
node crates/sketch-wasm/tests/protocol-compatibility.ts artifacts/web/sketch_wasm.wasm
```

The existing `check-wasm` entry point also runs the new Node fixed-fixture replay, so the
current CI path exercises it. Task 016/017 migrates this entry point to Node orchestration
and coverage discovery. Cargo integration tests are discovered by workspace test commands.

## Current boundary and known gaps

The [C header](../../crates/sketch-ffi/include/sketch_ffi.h) is authoritative for ABI 2.
A handle has one owner and all access/free is serialized; use on different threads is
allowed only serially. No live pointer validation can make a freed handle safe. Borrowed
input lasts one call. Output memory belongs to the caller; inputs/outputs must not alias.
Copy-size query returns required capacity; insufficient capacity returns 3 without partial
commands/bytes. Caught panic returns 6; discard the handle. Allocation failure/abort is not
a recoverable caught panic. The core remains free of unsafe code.

[Wasm1](../../crates/sketch-wasm/WASM.md) handles/buffer tokens are instance-local u32s,
not reused. Use one instance on one JS thread without reentry. Input is copied during a
command; output belongs to the editor and is invalidated by the next command/free (except
invalid handle/input-token rejection). Copy output and reacquire `memory.buffer` after
possible growth. u64 values use BigInt. Trap means discard the instance. These trusted
memory APIs are not a hostile-plugin isolation boundary.

Current export requires idle state, synchronously encodes committed content and returns
an editor-local revision. Revisions do not repeat across new/import/load in one handle;
undo/redo retains revision identity. Mark-saved rejects mismatches but does not carry a
session/request token and cannot enforce cross-handle/destination write order. Native
save/load APIs are synchronous. [Target contracts](concrete-contracts.md) introduce owned
snapshots/session tokens/storage ordering in later tasks; none exists by documentation.
Current generic Rust `Editor` does not enforce viewer capabilities. Separate Swift viewer
facades are not proof of VIEW-001. Source policy validates metadata ordering, not transport
authentication or content hash verification.

## Native interaction preservation matrix

Static audit at the capture revision by ui_ux_designer; no runtime UI was exercised for this
matrix. File references name the source of each policy. Extract each host's current policy
first; unification requires a deliberate behavior change and native acceptance tests.

| Host/source | Resize from opposite anchor | Zoom/navigation | Escape selection behavior |
| --- | --- | --- | --- |
| macOS `apps/macos/Sources/NativeSketch/NativeSketch.swift` | Signed diagonal projection, denominator>=.001, scale>=.01 | .1–16; ignore zoom while active | Cancel active; clear selection only if initially idle |
| UIKit `apps/ios/Sources/NativeSketch.swift` | Same projection/clamp | min(current,1e-15)–16; cancel before navigation | Cancel and clear selection |
| Windows `apps/windows/App.cs` | Radius from actual press, radius>=.001, scale>=.01 | .05–32; ignore zoom while active | Cancel, retain selection |
| Android `apps/android/app/src/main/java/dev/nativesketch/MainActivity.kt` | Bottom-right handle only, top-left anchor; radius>=.001, no .01 scale clamp; Rust rejects zero | min(current,.001)–8; cancel before navigation | Cancel, retain selection |
| Linux `apps/linux/src/main.rs` | Radius>=.01, scale>=.01 | .05–32; cancel before navigation | Cancel, retain selection |
| Web `apps/web/src/main.ts` | Radius>=.01, scale>=.01 | .05–32; cancel before navigation | Cancel, retain selection |

All hosts cancel active work and retain the document center when the
canvas size changes. Projection versus radial is observably different: an original diagonal
(10,10), dragged to (-10,-10), clamps to .01 under projection but yields scale 1 radially.
Do not conceal that difference behind a single new `ResizeStrategy` default.

Native focus policy precedes command routing: macOS NSTextView wins undo/redo and canvas
keyDown is canvas-owned; UIKit disables its keyCommands with a modal/text field/text view;
Android excludes EditText; Windows routes canvas keys from canvas and excludes TextBox for
CtrlUndo/Redo; Linux installs canvas-only key controllers; web guards editable elements and
dialogs. Core rejects undo during active interaction, while hosts may explicitly cancel
before invoking undo. Generic core cancel must not absorb host Escape-selection policies.

Known uncovered cases: web cancels on window blur, not canvas blur; its resize handler
cancels ink but retains contact/pair navigation bookkeeping. Do not assert full CANCEL-001
coverage from static handlers or synthetic replay. Native text-field behavior FOCUS-001
still requires actual UI tests on each host.

## Capture/check evidence

On macOS 27.0.1 (26A434), Rust/Cargo 1.99.0, Node 24.12.0, the task 015 fixture change passed
all 32 Cargo tests (26 existing + 6 new), strict workspace Clippy, rustfmt, the existing
native/Wasm conformance runner, and all 20 frozen Wasm request/response pairs. No runtime
source changed. New formatted test/script sizes are 128/83/50 physical lines. Native UI,
other OS ABI execution, allocation/panic fault injection and physical input were not run
for this fixture change. The integrated commit is recorded by the lead's task evidence.

SHA-256 capture identities:

```text
portable-v1.nsketch (216 bytes)
5b45bfd7215c21206aa30401b63f1b336ba90dc95549138103f2387a4dc9a352
protocol-v1.json
bce5df47fad5b83d0846d6d14074314d517e56e61a900ae896b82cfb41ff91dc
artifacts/web/sketch_wasm.wasm (release build, not committed)
119d7a2e95a2036020a775610e7f783823573ccc0569dea7f817fcbae908d2b6
```
