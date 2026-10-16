# Executable coverage evidence

Task017 foundation discovers actual Cargo test binaries and Node runner events, validates
feature/discovery/execution JSON schemas, joins exact identities, and renders an HTML/JSON
acceptance-by-platform matrix. It does not install missing native drivers or certify native
input from core tests.

## Commands

- `node scripts/test-evidence.ts`: compile/list Cargo tests, validate mappings, generate
  `artifacts/evidence/coverage.html` and JSON. Success means a valid report, **not** passing
  coverage. Unexecuted required cases remain `missing`.
- `node scripts/test-evidence.ts --run`: also execute each discovered Cargo test exactly,
  retaining test output and all result identities in `results.json`.
- `--node scripts/tests/evidence-report.test.ts`: additionally execute/discover this Node
  file through a structured reporter. Repeat for more files. No source scanning.
- `--require COMPAT-001,COMPAT-002`: fail unless every platform requirement for these exact
  acceptance IDs passes. Unknown IDs fail. `--gate` requires **all** registered rows and
  therefore currently fails because documented future-platform and behavioral gaps remain.
- `--input artifacts/evidence/macos/discovery.json --input artifacts/evidence/macos/results.json`:
  import native envelopes from the current source. Native artifact references are relative
  to their producer JSON directory, must exist, and must stay inside `artifacts/evidence`.

For a scoped native run, `scripts/test-e2e.ts` accepts repeated
`--only xctest:NativeSketchUITests:ClassName/testMethod` options. Selection never enables
foreground execution: `--interactive` still requires an explicitly reserved desktop.
The runner enumerates the complete test target first, rejects unknown/duplicate IDs,
retains `selected-tests.json`, and reports outcomes only for selected tests. Unselected
cases remain unexecuted, not passed or blocked by an invented outcome. Gesture cases
identify their additional native CGEvent/AX route separately from XCTest-only journeys.

All malformed mappings fail independently of scope. `--require` selects acceptance IDs,
not platforms: it deliberately cannot hide missing platforms within one acceptance case.
Partial/planned automation and explicit gaps cannot pass even when mapped tests pass.
Each requirement may carry a complete `coverage` override with `automation`, `testIds` and
`gaps`; absent overrides inherit case defaults. This lets macOS automation be implemented
without claiming implementation on other platforms.
A failed attempt followed by pass is `flaky`; all attempts remain in the report. Stale
source, binary mismatch, invalid environment, skip/blocked/unsupported cannot pass.

Source identity includes tracked diff and nonignored untracked bytes; clean commits have
null dirty digest. Exact discovery/execution source identities must match. Imported older
results are visible as stale; older discovery is rejected. All native subject/test bundle
hashes remain separate. Evidence modified by another writer during a run is rejected.

Node runner suite/hook failures invalidate passing leaf evidence. Fatal file/process
failures abort discovery rather than creating a false passing file test. Duplicate declared
Node names within a file are rejected; use stable unique names. Cargo ignored tests remain
skipped. Cargo package and target names come from metadata and compiler output.

The executable validator implements only the JSON Schema keywords used in the checked-in
schemas; this is not a general JSON Schema service. Cross-record rules are additional.
Failures/skips are data, not a reason to alter manifests or auto-update expectations.

## Native mapping and absent host discovery

BRUSH-001 and FILE-001 map only their macOS/native-e2e requirements to
`xctest:NativeSketchUITests:EditorJourneyTests/testDrawSaveReopen`. The actual journey passed
at source `50dbeef880a589ee8a064d3dc18d4840aeadd793`; its historical artifacts are recorded in
[the native evidence](../evidence/017-macos-e2e-foundation.md). Mapping this executable ID
records automation existence, not a current-source execution pass. The ordinary background
report has no native discovery and therefore shows those rows as `missing-discovery`.
It never opens an app. Current native discovery and results must be imported explicitly.

Discovery may declare `scopes: [{runner, platform, level}]` only after completed actual
runner enumeration, including successful empty enumeration. Existing discovered tests
also establish their runner/platform/level scopes for backward compatibility. Producers
must provide the complete enumeration for a declared scope; filtered test execution does
not establish enumeration completeness. Blocked setup and prepare-only builds omit scopes.
No producer is granted successful enumeration merely by compiling or scanning source.

For explicit requirement mappings, an absent ID in an enumerated matching scope is
`invalid-mapping` and a report issue (including typos and zero-test completed enumeration).
An absent scope is `missing-discovery`: a nonpassing gap, but a valid background report.
Until that runner enumerates, the ID cannot be verified and no passing claim is possible.
Legacy case-wide mappings keep their strict undiscovered-ID error behavior. Discovered
IDs with the wrong platform, level or required pointer width are orphan mapping errors.
The full gate and acceptance-case gate still reject every missing platform requirement.

## Remaining scope

Native macOS E2E has its own runner and console/permission requirements. Setup blocked
before test enumeration produces empty discovery/results plus `blocked.json`, never an
invented executable test. Other host drivers remain assigned to port tasks. This runner
has local verification; it is not a configured remote CI job. No coverage percentage or
physical pen-to-screen timing is inferred from this matrix.

## Retrying without erasing failures

Use `--attempt 2` (or the next positive number) when rerunning Cargo/Node in the same
checkout. It loads existing `results.json` and appends the new attempts. Duplicate attempt
numbers fail; changed source stays stale rather than becoming a pass. Every completed
report run also stores immutable discovery/execution snapshots under `history/` before
rendering, so a fresh attempt1 cannot erase raw prior evidence. Native producers similarly
accept an explicit attempt number; import each separate native execution envelope to
preserve its history. Do not edit failed JSON to renumber or turn it into a pass.
