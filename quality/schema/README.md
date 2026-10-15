# Evidence interchange v1

Executable contract: `scripts/evidence/types.ts` and `validate.ts`; JSON Schemas describe
serialized records for producers/editors. Cross-record validation additionally requires
unique feature/case/test IDs and attempts, discovered mappings, applicable platform/level,
and matching source and binary identities. A schema-valid result is not a passing result.

Discovery: `{schemaVersion:1,source,tests:[{id,runner,level,platform,pointerWidth?,binarySha256,testBinarySha256?}]}`.
Execution: `{schemaVersion:1,source,results:[{testId,attempt,status,durationMs,timestamp,binarySha256,testBinarySha256?,platform,toolchain,device,environment:{valid,reasons},artifacts}]}`.
`source` contains full Git `revision` and `dirtyDiffSha256` (null for clean). Digest includes
tracked diff and sorted nonignored untracked paths/bytes. XCTest records require the test
bundle executable hash separately from the app executable hash. Cargo hashes the actual
libtest binary. Node hashes the test module, identifies Node version, and binds imported
source through the whole-source fingerprint; this is not an application package hash.

IDs are `cargo:package:target:function` for portable contracts,
`cargo:package:platform:target:function` for host-scoped Cargo packages,
`node:relativeFile:declaredName`, and
`xctest:bundle:Class/method`. Cargo discovery runs compiled `--list`. Node discovery is
combined with execution through structured runner events. Nested duplicate Node names
are rejected until callers provide globally distinct names within each file.

Artifacts must be relative, remain within `artifacts/evidence`, and exist when imported.
All attempts remain visible. Native setup blocked before enumeration writes empty
`tests`/`results` and a separate `blocked.json` diagnostic; it must not invent IDs.
Native missing discovery remains missing required coverage. Other platform adapters
remain assigned to port tasks, including visionOS automation limitations.
