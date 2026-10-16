# Android JNI checkpoint and handoff

Snapshot stopped at the owner's request to publish a draft PR on 2026-10-16.
This is additive transport work, not Android Activity adoption or merge readiness.

## Integrated source and evidence

- Rust worker `052c88dd7e1727c9b560d991ddaca118f9dd6617`, integrated as `9b93588`:
  bounded Rust JNI transport over the existing safe application bridge, fixed ownership
  receipts, preflight validation, request correlation and panic containment. Fresh independent
  review found no actionable transport defect and reran four focused tests successfully.
  Production exports exactly 13 JNI methods and excludes the isolated exception probe.
- Kotlin worker `00813d6`, integrated as `c677991`: pure Kotlin declarations, private direct
  buffers, transfer holders, worker handoff, reply ownership, real JVM tests and background
  evidence/packaging runners. Legacy Activity and C++ libraries remain intentional.
- Development runs passed 13 actual JVM/JNI cases (12 production and one separate pending
  exception case), two pure holder tests and Android quality checks. These preceded final
  reporter changes and do not qualify the final commit.
- Worker final strict TypeScript/Oxlint and nine focused reporting/publication tests passed.
  Exact final JVM rerun, Android ARM64/x86-64 APK execution and final Kotlin review are pending.

Frozen Rust production dylib SHA-256:
`c819b9bf941ead1e424b9765b8f6d4fc420f8d665630ed28998ac42eb57054ef`.
Separate test probe SHA-256:
`25a0c469d464778029eb8809fc377a8639dbd844ff879ee6b3c8b390a66447f5`.
Never load both libraries into one JVM or package the probe in an APK.

Raw local evidence is preserved under `artifacts/evidence/022-jni-rust-worker/` and
`artifacts/evidence/022-jni-kotlin-worker/`. It is ignored build/test output; repository
documents record its scope rather than pretending local artifacts are remotely hosted.

## Resume in this order

1. Review final Kotlin/runner commit. The reviewer had not verified closure of publication
   ordering, cancellation, failed XML retention, immutable attempt directories, exact compiled
   discovery/signature/export validation and packaging provenance when work stopped.
   The final worker reports metadata-first, execution-last publication with the shared
   single-record atomic helper; independently verify this, including failed identity reads.
2. Run the combined background integration gate, exact-commit real JVM qualification and
   dual-ABI additive APK build/inspection. Preserve failed attempts and qualify actual
   source/library/class/JVM identities. No Android runtime acceptance follows from compilation.
3. Map final runtime-discovered JNI test IDs into feature manifests. Rust host tests and
   JVM tests on macOS must not be labeled native Android E2E.
4. Implement the shared [copy-document policy prerequisite](../architecture/022-copy-document-policy.md)
   before migrating the Android Activity/provider coordinator. Freeze its typed/wire contract
   first. Then verify provider ownership, URI grants, recreation, native focus/input and Skia.

The next policy design proposes immutable application-level `FilePolicy` (bound document
with existing save-destination policy, or editable copy), constructor opcode 14 with a
56-byte payload and policy tags 0/1/2, and confirmed-copy save-purpose tag 5. Old opcodes
1/8 and purpose tags 1–4 stay unchanged. These are proposals, not implemented packets.

One key API decision: replace the consuming Rust `acknowledge_save(SaveCompletion)` method
with a borrow-aware `acknowledge_save(&mut Option<SaveCompletion>) -> Result<..., StorageError>`.
Rejection preserves the proof and exact session gate; success consumes once. Do not retain
a public consuming wrapper that could lose the sole copy completion. Update routing and
typed tests while preserving native-v1 ABI behavior. Put file policy types at application
level with compatible reexports, rather than making session storage depend on routing.

The private copy gate must span Choosing → Issued → validated owner Apply. Prepare bounded
committed snapshots and reservations before consuming a chooser or cancelling current ink.
Rejected copy proof must retain Ready payload/resource; existing generic stale-candidate
retirement is not suitable. Default desktop save behavior remains unchanged.

Task 024 remains open. Native UI/GPU checks, remaining platform migration, distribution
acceptance and physical latency are not completed by this checkpoint. No merge or release.
