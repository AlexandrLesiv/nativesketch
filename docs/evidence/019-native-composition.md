# Task 019 native composition evidence

## Scope

Mechanical extraction commit `2bfd891` replaced the 848-line macOS entrypoint with
composed native window, inspector, menu, canvas/input, document and legacy bridge objects.
The map is in `docs/architecture/019-native-composition.md`. This is the first migration
tranche, not completion of the typed-session bridge or Skia adoption. ABI 2 and the
historical Core Graphics painter remain. New/extracted Swift files are under 200 lines.

## Native execution

Four cases passed on `d800a6d1270f5feb620c2cd14581ef62afdb5630`, with clean source:

- `EditorJourneyTests/testDrawSaveReopen`: native width/color controls, mouse stroke,
  independent pixels/serialized geometry, save, quit/new process, reopen and Save As.
- `KeyboardShortcutTests/testSaveAsKeyboardBeforeAndAfterBinding`: fresh Command-Shift-S,
  cancel, ordinary Save, bound Command-Shift-S and cancel preserving filename/bytes.
- `KeyboardShortcutTests/testOneStrokeUndoRedoKeyboard`: one completed stroke removed
  by Command-Z and restored by Command-Shift-Z. At this revision the final bytes check
  used an already existing destination; a later test strengthening forces fresh output.
- `KeyboardShortcutTests/testFieldEditingKeepsCanvasState`: native field focus,
  selection/ink retained through field keys and Pen/Select unchanged. At this revision
  field Undo/Redo text changes and fresh serialization were not yet proven.

Hardware/toolchain: Mac14,10 arm64, macOS 27.0.1 (26A434), Xcode 27.0 (27A266a).
Input route is XCUIAutomation mouse/keyboard, not a physical pen or latency measurement.
Ten pure Swift oracle regressions also passed during preparation. Strict Swift formatting,
background build and headless integration smoke passed in the mechanical tranche.

Artifact root (local, ignored):
`/private/tmp/native-sketch-macos-composition/artifacts/evidence/native-four-cases`.
`identity.json`, `results.json`, `discovery.json`, `execution.log` and `journey.xcresult`
retain exact identities, runtime enumeration, assertions and screenshots/AX trees.
App SHA-256: `4cdc8062bf6ea812cc610122e5b348118fe93a1891304b86f8b09d52b537602e`.
Test SHA-256: `2eadd21f6939b03ed6379820a73e8a4a58914377a6fc1eee9a2d3da680d98e00`.

## Save As investigation

The original lowercase `s` plus Command/Shift menu binding remains unchanged.
The owner confirmed and disabled a conflicting system-wide Command-Shift-S shortcut.
True fresh-launch and bound-document keyboard tests then passed without Save warmup.
Do not describe this as an application shortcut binding fix. A synthetic nonpresenting
AppKit menu probe established only case-sensitive synthetic event matching; it could not
prove actual delivery. The attempted uppercase binding did not fix the external conflict.
Temporary event/action diagnostics were removed. The retained DEBUG, explicit-opt-in
native observation only reports activation/key-window/first-responder categories via AX;
it neither changes focus nor dispatches commands. No raw event logging remains.

Earlier failure artifacts are preserved in separate directories: `shortcut-baseline`,
`shortcut-fixed`, `shortcut-diagnostic`, `shortcut-application-delivery`,
`shortcut-delivery-control`, and `native-shortcuts-final`. The last includes an incorrect
String cast for an NSNumber radio-button value; correcting the oracle produced the valid
four-case run. No failed envelope was converted into a pass.

## Remaining coverage and architecture gaps

Direct runner AX focus access returned AXError -25211. No accessibility trust settings
were changed; the supported observational AppKit first-responder oracle is used instead.
VoiceOver speech, physical pen behavior, locale/IME, display transitions, failure recovery,
invalid/incomplete width focus refusal, and full FOCUS-001/HISTORY acceptance remain gaps.
The profile-menu screenshot still clips its bottom outside the main window; it is not
complete popup evidence. Designer review must state its actual inspection scope.
Shared typed dispatch, async storage, validated shared settings and removal of native
widget reads are subsequent agreed migration work. No performance claims follow from
this extraction. No merge or release is authorized by these results.

## Strengthened oracle verification

Independent review identified two insufficient assertions: reading an existing saved file
could pass even if Save did nothing, and retaining ink through field Undo did not prove
native field text Undo. Commit `458f9234e708fe053939eca55f9d702f9ad9d3dd` strengthens
these without changing production behavior. All four native cases passed at this clean
revision, recorded in `artifacts/evidence/native-oracles-retry` (attempt 9).

Undo now writes a previously nonexistent destination and checks the document header's zero
stroke count. Redo writes a second nonexistent destination and compares full original bytes.
After field keyboard actions, another fresh destination must exist with identical committed
bytes. Field Undo must change text from `12`; field Redo must restore `12`. These assertions
passed with the original native menu/responder implementation; no speculative routing change
was introduced. Their scope remains these tested paths, not complete FOCUS-001 coverage.

App SHA-256: `dc7585cdf5ebe298110973b876f4f1fa19074aef3271c0456b25891c484ccfe8`.
Test SHA-256: `3fc40ab333c3a4651ad751e77170b591ec750061901269f283f8d3584d01b31f`.
The device, SDK and input route match the earlier four-case run. Preparation and oracle
regressions passed; strict formatting passed. Attempt 8 in `native-oracles-verified` timed
out enabling XCTest automation before enumeration. It remains blocked, with no discovered
or executed tests. The console was confirmed unlocked and owned by the runner user. One
bounded retry succeeded without settings, permissions, daemon or test changes.

The designer independently opened/cancelled fresh Command-Shift-S and observed canvas focus
return on the frozen d800 candidate. Inspector/ink/Save As screenshots were reviewed. Full
color-profile popup and keyboard-only traversal remain explicit inspection gaps. The designer
quit the disposable app before handing the host back for the strengthened tests.

## Native incomplete-width validation

Separate behavior commit `8973b71e950f11355cb4de19cf4997ac4feff01a` adds a native
field-edit coordinator: unfinished/rollback text only, native formatter acceptance,
popover close veto, Escape restoration, and successful native edit/focus transfer before
canvas input. The inspector no longer silently clamps unfinished width text. This is a
native FOCUS-001 case; Rust remains the intended authority for validated brush settings
in the subsequent typed command migration.

All five enumerated native cases passed on that clean revision, first execution attempt
10 at `artifacts/evidence/native-field-validation`. The new case is
`NativeFieldValidationTests/testIncompleteWidthRefusesInkAndEscapeRestoresValue`.
It clears the actual width field and drags the canvas: the popover, empty text and native
field focus remain, with zero ink. Escape restores 6. Correcting to 12 and drawing produces
one saved stroke with serialized width 12; one Undo removes it and no extra Undo remains.
The four strengthened existing journeys also passed. Preparation/pure oracle regressions
and strict Swift formatting passed. Hardware/toolchain/input route are unchanged.

App SHA-256: `0fafa4399c3da19ee03a7cdd41699c925415ca23e3166f13dfa42baa40923ce8`.
Test SHA-256: `050602c894e042a696823b7ea8712fa238913a8c316172a5d965456b7147e7f0`.
The local attachment `attachments/24278AFE-2525-4A30-8B07-3B9280025B20.png` shows the
empty focused width, blank canvas and explanatory status. XCTest app-level screenshots
include background desktop content on this host: raw screenshots/recordings remain local,
ignored evidence and must not be published as product illustrations. Future screenshot
collection should prefer the exact test-owned window or popover. This run does not prove
VoiceOver speech, IME/localized number entry, all invalid ranges or all field navigation.
The subsequent typed presenter must also preserve active text/caret through async refresh.
