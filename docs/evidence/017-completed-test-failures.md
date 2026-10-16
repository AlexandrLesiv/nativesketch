# Completed XCTest failures versus infrastructure failures

Worker `3adfc10` / integration `a6637a7` repairs result classification without rerunning
the native interface. Previously any nonzero xcodebuild exit invalidated passing peers,
including a fully completed run with ordinary test failures.

The runner now retains typed process status and the original nonzero command failure.
Only a normal exit 65 with one complete expected Test action, matching terminal selected
tests, consistent counts/timestamps and attributed issue multisets can be classified as
completed test failures. Unknown identities, missing fields, extra actions, runner errors,
timeouts, cancellation and output-limit failures remain invalid infrastructure evidence.
Loaded-bundle provenance and unchanged source are still required for every passing peer.
Overall native command exit remains nonzero when tests fail.

Reduced fixtures preserve the actual historical task 020 attempt's seven passes and four
failures, including its real `Uncategorized` issue type. They remove private machine/user
metadata and use synthetic source locations. Raw historical artifacts are not rewritten.
Attributed teardown issues remain failed tests; global finalization errors cannot certify
otherwise passing leaves. Exact aliases replace ambiguous suffix matching.

At clean worker `3adfc10`, both author and independent reviewer passed all 81 Node tooling
tests, strict TypeScript, type-aware Oxlint, Oxfmt and source-size checks. The primary also
passed the focused classifier/process/cleanup tests at `a6637a7`. Tests include real child
process failures, historical structured-result replay and malformed/ambiguous negatives.
Evidence lives in `artifacts/evidence/xcresult-classification`; the new registry cases are
`EVIDENCE-XCTEST-001`, `EVIDENCE-PROCESS-001` and `EVIDENCE-TEARDOWN-001`.

No current native UI, runner finalization failure or desktop interaction was rerun.
This is reporting correctness evidence, not application acceptance or merge approval.
