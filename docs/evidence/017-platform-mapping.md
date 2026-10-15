# Task 017: platform-specific native coverage mapping

Writer revision `e8ca3617868a4f7dfe83363c5e2d7a271f274434`, integrated as `66dfcc2`.
This maps macOS BRUSH-001 and FILE-001 to the actually discovered XCTest journey without
claiming automation on the other eight required platforms.

A requirement may supply a complete automation/test-ID/gap override of its case defaults.
Completed executable enumeration scopes distinguish a missing host run from a dangling
mapping: no native enumeration is `missing-discovery`; an expected ID absent from a
completed matching enumeration is invalid and fails validation. Preparation-only or blocked
setup does not count as completed enumeration. Existing case-wide mappings remain strict.
Unknown results, duplicate IDs, incompatible platform/level/pointer width, old source,
binary mismatch and retained failed retries still cannot pass.

At clean integrated `66dfcc2`, pinned Node 24.22.0 on macOS arm64:

- Full background check passed 50 tooling tests, 51 Rust tests, format/lint/types,
  dependency/source-size checks and native Swift linking/headless interoperability smoke.
- Actual Cargo execution and scoped COMPAT-001 through COMPAT-005 report generation
  passed: 152 requirements, 11 passed, no mapping errors, overall coverage incomplete.
- The two mapped macOS native requirements report `missing-discovery` in this background
  run; the report does not launch an application to obtain a pass.

A fresh reviewer independently ran 23 focused evidence/runner tests, strict types, lint,
format and source/dependency checks. Actual background report generation without execution
correctly produced zero passes, preserved all 152 rows and had no mapping errors. Native
required-case gates failed as expected; the historical attempt-8 discovery was rejected as
stale. Independent override/scope probes retained all platform requirements and rejected
enumerated typos, empty completed scopes and incompatible mappings. No findings remained.

Raw writer logs are under ignored `artifacts/verification/mapping-worker/`; integrated logs
are `artifacts/verification/mapping-integrated.log` and `mapping-integrated-evidence.log`.
The historical native journey remains valid evidence for `50dbeef`, not for this source.
The next relevant native commit verification can supply fresh native envelopes for the
mapped macOS rows. Other journeys/platforms and owner acceptance remain pending.
