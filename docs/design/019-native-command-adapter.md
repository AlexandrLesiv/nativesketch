# Native command adapter acceptance

Agreed from the designer and native owner planning review at `6567c4e`. This is the
019D/E acceptance target, not evidence that these journeys already pass. Native controls,
shortcuts and future agent adapters share the Rust ApplicationSession command boundary.

## Interaction and ownership

- AppKit presents segmented tools, numeric width/stepper, color well, selection scale,
  menus, panels and sheets. Rust supplies committed values and enabled/disabled reasons.
- The native field owns unfinished text, caret, selection and text undo. Unrelated
  projections, including asynchronous save completion, must not overwrite active edits.
- Before canvas input or explicit file/navigation intent, native editing must finalize.
  Valid edits dispatch the shared setting command first. Invalid text keeps focus, shows
  guidance and blocks the intent. Escape restores the prior committed value. A rejected
  Quit returns native cancellation rather than leaving termination indefinitely deferred.
- Native text Undo/Redo, Delete, selection, letters and Space keep first-responder priority.
  Preserve Cmd-S, Cmd-Shift-S, Cmd-O, Cmd-N, Cmd-W and Cmd-Q. Accepted file commands cancel
  drafts through Rust policy; rejected commands must not erase them.
- Canvas reads the Rust projection, never inspector widgets. Viewport mapping, native
  event provenance and cancel-before-remap remain adapter responsibilities.
- Production Swift imports only the new session/runtime C module. Legacy ABI2 conformance
  stays in dedicated compatibility fixtures; no production fallback to its editing or I/O.

## Asynchronous presentation

| State | Native presentation and behavior |
| --- | --- |
| Choose destination/source | Correlated NSSavePanel/NSOpenPanel sheet; Cancel preserves content and binding |
| Writing | Nonmodal Saving status; subsequent drawing remains available after snapshot capture |
| Pending Save As | Old title/binding until confirmed acknowledgement; another Save As disabled; old-bound Save only when Rust allows |
| Save failure | Try Again, Save As, Cancel; retry is a new intent and snapshot, never replayed success |
| Written with warning | Say the file was written but durability is unconfirmed; never claim it was unchanged |
| Navigation awaiting writes | Finishing Save status and Cancel Navigation; accepted disk writes still settle |
| Dirty navigation | Save / Cancel / Don't Save native sheet; replies carry identity and authorized revision |
| Reading | Opening status and cancellation; failure or obsolete completion preserves current content |
| Close approved | Final issued confirmation, then synchronous close before further input |
| Quit pending | terminateLater and exactly one platform reply; no UI waits or forgotten grants |
| Recovery | Native list: Open Recovered Copy, Later, Discard; destructive confirmation defaults to Cancel |

Only one sheet owns a session's current presentation. Correlate each sheet and failure to
its issued effect/operation; obsolete results are logged without stealing focus or stacking
new sheets. Focus restoration belongs to the presenter and must verify that the originating
responder, window and session still exist. Ordinary save completion never moves focus.

Recovery inventory comes from the runtime, with bounded local display metadata, validity/
fallback and capacity information. Use NSTableView with selection-dependent actions.
Opening follows normal unsaved-navigation protection and creates a dirty unbound copy.
Later/Escape preserves candidates. Opening alone does not resolve or delete a checkpoint.
Explicit confirmed discard affects only the selected candidate; resolution after save must
be correlated to the exact recovery identity/generation, not an optimistic UI assumption.

## Required native journeys

These proposed IDs must be registered only with accurate implementation/gap state.

| Acceptance | Decisive assertions |
| --- | --- |
| NATIVE-ROUTING-001 | Equivalent menus/shortcuts/controls reach the shared dispatcher and produce equivalent projection/output; pair UI with bridge tests |
| NATIVE-FIELD-REFRESH-001 | Delayed save completion preserves unfinished width text, caret, focus and text undo; corrected value produces fresh saved geometry |
| NATIVE-ASYNC-SAVE-001 | Hold admitted save A, draw B natively, release A: disk A, document dirty; next Save writes B |
| NATIVE-ASYNC-SAVEAS-001 | Old binding/title pending; success adopts new destination, failure/cancel retains old; changed-content Save proves destination |
| NATIVE-SAVE-FAILURE-001 | Real disposable write failure and controlled post-replace warning have accurate feedback, retention and successful new-intent retry |
| NATIVE-OPEN-ORDER-001 | Open cannot overtake accepted Save; corrupt/oversized input and obsolete read preserve current content/binding |
| NATIVE-NAVIGATION-001 | New/Open Save/Cancel/Don't Save, outstanding writes and newer edits never bypass current authorization |
| NATIVE-CLOSE-QUIT-001 | Cmd-W/Q with writes/failure/cancel remain open when cancelled; success exits normally and saved content reopens |
| NATIVE-STALE-EFFECT-001 | Late cancelled result cannot replace content, change title, steal focus or open an obsolete sheet |
| NATIVE-RECOVER-COPY-001 | Later preserves recovery; menu reopens list; recovered copy is dirty/unbound and first Save needs a destination |
| NATIVE-RECOVER-MANAGE-001 | Corrupt-newest fallback, empty list, capacity retention and selected-only confirmed discard |
| NATIVE-ASYNC-FOCUS-001 | Save completion and cancelled sheets preserve/restore appropriate focus; resize/deactivate cancels before remap |

Keep all five existing native regressions. Test barriers/faults operate at the real worker
boundary, scoped to disposable fixtures and DEBUG-only configuration. They must not mutate
documents, fake successful completions or bypass actual native command admission. Record
injected scheduling separately from actual filesystem faults and process-kill evidence.

Use stable accessibility IDs for status, cancellation, recovery and error actions. Announce
important failures/completions selectively, never each poll/sample. Inspect native defaults,
Escape/Return behavior, minimum and larger sizes, light/dark, focus and disabled states.
Controls and cancellation remain reachable through native overflow/scrolling. AX inspection
does not prove screen-reader speech. Existing color-popup capture, AX Disabled reporting,
keyboard traversal and physical-pen limitations remain explicit until actually verified.
