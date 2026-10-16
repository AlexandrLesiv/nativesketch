# Task 023: bounded core fuzz execution

Source `f8bb00b00d0a6c44f02f291a7bf8aeb474925e53`, clean, macOS ARM64,
2026-10-16. Node 24.22.0, cargo-fuzz 0.13.2, libfuzzer-sys 0.4.13 and pinned
nightly-2026-10-15 (rustc `7df81381d027710708209f36b98d11b1c88f8d2e`).

Both targets completed 25,000 generated inputs, then exited 86 with the same 56-byte
LeakSanitizer report. **Both executions and the aggregate are failed.** Completion of
input generation is not a passing sanitizer result. No core oracle assertion failed in
these runs; this does not prove that no application defect exists.

| Target | Binary SHA-256 | Recorded result |
| --- | --- | --- |
| document_codec | `a911f27baf8583a09fca81615241d3129c31e240063ccc89b9f54aeca54b8cbf` | Failed: shutdown leak after 25,000 inputs |
| command_sequence | `75b10ad38dcce195f7f2ed658e0d0feab89109f2a9d2b04feeefda631292f9c2` | Failed: shutdown leak after 25,000 inputs |

`artifacts/evidence/023-fuzz-attempt-3` retains exact commands, instrumented core
fingerprints, binary hashes, corpus, logs, generic discovery/results and scoped HTML/JSON
coverage. The scoped report has no mapping issues and correctly marks FUZZ-CODEC-001 and
FUZZ-COMMAND-001 failed. The separate whole-project report reports seven unenumerated browser
tests because this invocation did not discover browser tests; it is not a complete gate.

Attempt 1 failed before execution on the pinned Cargo fingerprint layout; attempt 2
failed on the document target's sanitizer exit. Both directories and logs remain intact.
The runner now executes independent remaining targets after an ordinary child exit failure,
retains each result, and fails the aggregate. Source mutation, cancellation, timeout and
other environment failures stop execution. Fresh review at `f8bb00b` found no actionable
issue; three Node runner tests, additional reviewer fail-stop probes, strict TypeScript,
Oxlint, Oxfmt and whitespace checks passed. This is runner review, not fuzz acceptance.

## Isolating the failure

`artifacts/evidence/023-fuzz-runtime-probe` contains an independently compiled empty Rust
libFuzzer target with **no NativeSketch dependency**. Its binary SHA-256 is
`228560244d46271b4be47313b17524eee2d129d797fe740b0db1a5b56cb12561`.
With the same sanitizer settings and 1 GiB RSS monitor it also exits 86 on the same
libFuzzer/libc++ detached-thread allocation. A diagnostic one-input control with that RSS
monitor disabled exits zero. The vendored runtime's `StartRssThread` creates and detaches
this thread; its allocation stack is present in both application and empty-target logs.

This isolates the observed report to the test runtime on this host. It does not establish
that every future leak report is spurious. The zero-RSS control is **not an accepted test
configuration**. Production lane limits and leak detection remain unchanged; no suppression
or weaker threshold was introduced. A validated runtime repair or separate supported host
is required before this lane can pass. Do not retry unchanged runs indefinitely.

The [harness documentation](../../fuzz/README.md) names the exact asserted properties.
Native cancellation, callback replacement, pressure ordering, one-operation undo and
prediction exclusion require their own tests; generating those commands is not proof.
