# macOS component extraction evidence

The mechanical extraction was reviewed at worker commit
`2bfd89177b6503670daebb7128c3735fa2e47e55` (base `62d54e8`) and integrated as
`0cdb2e0`. Integrated verification used clean `91efb03` before the later documentation
refinements. This milestone does not complete task019 or fix the Save As shortcut.

The 848-line entry file is now23 lines. Sixteen Swift source files have cohesive named
responsibilities and contain10–147 physical lines. See
[component boundaries](../architecture/019-native-composition.md). ABI2, document bytes,
current native defaults, synchronous legacy persistence and the temporary Core Graphics
painter are preserved. Canvas still reads settings through the documented legacy seam;
the next typed-session migration must remove that dependency.

## Verification and independent assessment

Full background `scripts/check.ts` passed at91efb03:50 Node tests,76 Rust tests,
format/lint/type/source-size/dependency checks, Swift build and headless Swift/Rust
integration. Log: `artifacts/verification/019-extraction-integrated.log`.
This smoke check uses injected events; it is not native event-delivery evidence.

A fresh reviewer independently inspected the exact extraction diff, built it in a
separate disposable directory, ran strict Swift formatting and the headless integration.
No actionable extraction regressions were found. Review covered single handle ownership,
main-actor confinement, retained menu targets/validation forwarding, unowned component
back-references and preserved field-editor undo/redo branches. The independent binary
SHA256 is `76564c4be012693cab6f6ec6aff4f6527b02cb21e4e77cf98b6c73aaaafb946f`.
Machine evidence: `artifacts/verification/019-independent/review-evidence.json`.

## Native acceptance and remaining work

Separate native keyboard tests reproduced the existing Command-Shift-S failure.
A speculative capitalization change also failed and was reverted in the worker branch;
it is not integrated as a fix. Opt-in bounded diagnostic development is continuing.
The full native run is not accepted when any required test or finalization fails;
individual passing raw test leaves are not a substitute for a valid complete envelope.

Typed Rust session adoption, asynchronous storage/recovery, structured native diagnostics,
field/focus and accessibility acceptance, designer review and the actual shortcut
resolution remain task019 work. Production Skia is task020. Other native platform files
retain their explicit extraction debt. No main merge, release or performance claim.
