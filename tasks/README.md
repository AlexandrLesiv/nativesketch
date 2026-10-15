# Backlog

Run `./scripts/board` to view status and ready tasks. `board.json` contains IDs,
dependencies, roles, and links to task briefs. It does not launch agents or spend tokens.

The implementation base is `codex/github-pages@712059d5`. The owner excluded
`codex/renderer-results` from new implementation ancestry. Task 002 is historical review
on that separate branch; Skia is selected. Ports no longer wait for another comparison.

Follow [the detailed plan](../docs/refactoring-plan.md): 015 contracts -> 016 Node/tooling
-> 017 E2E evidence -> 018 Rust -> 021 diagnostics/recovery -> 019 macOS -> 020 Skia
-> 022 host contracts -> individual 004/009-013 ports and 023 hardening -> 024 acceptance.
These are future implementation tasks, not completed by this documentation change.
015 is ready; downstream tasks wait for prerequisites and owner acceptance. Task 003 keeps
recorder/replay scope and 014 keeps network/account scope. Native UI/GPU jobs remain exclusive.
Run existing scripts until task 016 proves Node parity and updates command references.

The repo setup is separate from task 001. A working blank shell is not completion of the
first drawing milestone. The owner accepts a task before its board status becomes done.

