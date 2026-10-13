# Foundation validation — 2026-10-14

The source tree in the initial foundation commit was checked both in staging and at
`/Users/oleksandr/projects/native-sketch` on the local Apple Silicon Mac.

Passed:

- `./scripts/check`: task-board validation, Rust formatting and clippy, Rust test targets,
  Swift build, app bundle creation, and the Swift-to-Rust ABI smoke check.
- App bundle property-list validation and inspection of native library dependencies:
  Rust is statically linked, with no separate Rust dylib required at launch.
- Launch of the published development app. Its native window and placeholder text were
  inspected using accessibility information and a screenshot.
- Codex 0.162.0-alpha.17.2 configuration-load diagnostic: project context parsed with
  `gpt-6-astra` selected. Official custom-agent schema was used for all seven role files.
- A fresh read-only agent review of the staged source found no actionable setup issues.
  The published source was compared byte-for-byte with that reviewed tree.

Toolchains used: Rust 1.99.0 and Swift 6.4. The installed Command Line Tools emitted
two missing library-search-directory warnings; linking and launch still succeeded.
SwiftPM's nested build sandbox required host-approved execution from this Codex session.
No sandbox bypass was added to the project's build scripts.

Limits: Rust currently has zero behavior tests because drawing semantics are not
implemented. These checks establish scaffolding and native linkage, not drawing quality,
pen responsiveness, Windows compatibility, or support for older toolchains/OS releases.
Custom role configurations have not yet been exercised in a complete feature run.
No remote CI, remote repository, release signing identity, or merge protection is configured.
The broader machine diagnostic includes unrelated sandbox/network checks; only its
configuration-load result is reported here.
