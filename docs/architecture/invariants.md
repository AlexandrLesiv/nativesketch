# Invariant register

Normative target for extraction and acceptance. These IDs do not assert that every mapped
test exists: task 015 inventories current enforcement and task 017 creates executable
traceability. Tests must prove observable behavior, not just repeat constants.

| ID | Rule | Enforcement owner | Minimum evidence |
| --- | --- | --- | --- |
| INK-001 | Cancel leaves committed content/history unchanged | Separate Rust draft state | Begin/append/cancel; compare bytes/history |
| INPUT-001 | Invalid batch cannot partially mutate | Validate entire batch before apply | Invalid trailing sample after valid prefix |
| INPUT-002 | Actual samples ordered; missing pressure stays missing | Input validation and adapter contract | Rebatch/coalescing/correction/device fixtures |
| INK-002 | Predictions never enter saved/committed data | Separate actual/predicted buffers | Commit/export while predictions exist |
| INK-003 | Brush snapshot fixed for active stroke | InkSession captures settings | Change settings during draft; next stroke differs |
| UNDO-001 | One committed stroke/transform creates one undo entry | History records only at commit | Many updates followed by one undo/redo |
| UNDO-002 | Cancel/no-op creates no history; active undo keeps defined rejection | Transition validation | Invalid/no-op sequences leave state intact |
| SAVE-001 | Successful older save cannot mark newer edits clean | Session/revision/request token | Save A, edit B, complete A; B remains dirty |
| SAVE-002 | New/import invalidates old save completions | New document-session identity | Same revision number in different sessions |
| SAVE-003 | Older write cannot overwrite newer content at same destination | Storage coordinator ordering | Delayed/reordered write fixtures |
| LOAD-001 | Invalid candidate preserves current document | Decode/validate before replace | Corrupt, unsupported, oversized input |
| FOCUS-001 | Native text editing wins over canvas shortcuts | Native focus routing | Undo/Delete/P/V/Space in focused field |
| CANCEL-001 | Cancel before coordinate remap or gesture ownership change | Host input lifecycle | Resize/capture loss/deactivation/second touch |
| VIEW-001 | Read-only sessions reject mutation through the API | Rust capability + narrow native facade | Programmatic mutation attempts rejected |
| RENDER-001 | Renderer cannot mutate document | Read-only scene contract | Dependency/API review and equivalence tests |
| CACHE-001 | Cache eviction does not change final content | Disposable caches | Cached/uncached and context-loss comparison |
| ASYNC-001 | Stale callbacks cannot affect replacement session/surface | Generation validation | Close/import/resize while work pending |
| SOURCE-001 | Source fallback preserves document identity and revision policy | Rust source policy + native transport | Phone-only, stale/conflicting, corrupt/late responses |
| FFI-001 | Unique handle ownership, serial mutation, no cross-FFI unwind | Typed native wrappers and Rust boundary | Real-consumer lifecycle/error tests |

## Enforce in layers

Validated types/private constructors prevent invalid values where possible. ActiveInteraction
is one enum, not independent booleans. Validate untrusted file/FFI/input/agent boundaries in
release builds. Debug assertions supplement validation; never replace it.
Each invariant links to owning symbols and discovered test IDs once task 017 lands.
Property tests exercise valid operation sequences; fuzzing probes malformed inputs and
resource limits. Retain failing seeds and minimized fixtures. Native focus/capture behavior
requires native E2E, not merely a synthetic Rust event.

## Compatibility guardrails

Preserve current new-document history reset, undo rejection during active work, correction
timestamp rules, topmost selection and file-copy dirty semantics. Cross-host resize math
and zoom policies differ: document each difference before intentionally unifying it.
Native save dialogs, imported document replacement and provider error handling keep their
accepted cancellation behavior. Download dispatch is not confirmed persistence.
Phone transfer queued is not proof of receipt/display, and sending a copy is not saving.
