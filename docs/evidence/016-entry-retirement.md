# Task 016: board and check entry-point retirement

The retained Python `scripts/board` and Bash implementation of `scripts/check` have been
retired. `node scripts/board.ts` is the board command. `./scripts/check` remains an
executable, three-line Node compatibility name that imports the authoritative `check.ts`;
it contains no independent orchestration. Use pinned Node 24.22.0 on PATH.

Before deletion, the Python and TypeScript board commands produced identical listing and
`--check` output when invoked from outside the checkout at clean `93e8d6c`. Exact output
and identity are retained in `artifacts/evidence/016-board-retirement-parity.json`.
Existing malformed-board/dependency/acceptance-lifecycle tests remain in the Node suite.

The authoritative full Node background gate passed at `f28da29`, including actual C/Wasm
export consumers, production macOS linkage/pre-AppKit smoke, strict Swift formatting and
17 GPU-free Swift tests. This is background evidence, not native UI acceptance.

The new compatibility test invokes a copied executable entry from an unrelated directory
against a disposable TypeScript module. It verifies exact argument forwarding (including
spaces, punctuation and Unicode) and exit-17 failure propagation. It launches no GUI and
does not substitute a fake application result for a real build. The existing production
Node gate remains the actual build/test implementation.

Other legacy platform scripts remain pending behavior-specific parity. In particular,
simulator builds do not prove device installation, native run commands, companion copying,
or Windows process/runtime behavior. Do not remove those fallbacks merely because an
individual cross-compilation passed. Historical evidence retains the commands actually run.
