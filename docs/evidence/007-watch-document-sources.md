# 007: Watch source adapters evidence

Date: 2026-10-14. Branch `codex/watch-sources-desktop`, base `03aa825`.
Main is unchanged. This is a local first-stroke prototype, not a release.

## Implemented behavior

An independent watchOS viewer (`WKSupportsRunningWithoutiOSApp`) keeps optional iPhone
integration. Native Details/Library/Refresh controls select explicitly shared copies.
Cached drawings open immediately; refresh prefers iPhone with a bounded request deadline,
then a configured HTTPS server for the same published drawing identity. Phone-only drafts
cannot fall back to a server copy. Unsolicited shares do not replace another selection.
Changed geometry during navigation is staged for Apply Update. Rust owns source identity
and revision acceptance through an additive ABI 2 function; Swift owns native transport,
SHA256 checks, request correlation and atomic cache publication. All candidate bytes pass
Rust document validation before becoming visible.

No production storage or authentication provider exists. The bounded HTTPS adapter is
implemented and tested against controlled transport fixtures; production remains explicitly
unconfigured. It currently supports public/developer data only. No credentials or fake
sign-in are shipped. See `apps/watchos/SOURCES.md` for the concrete service contract.

The phone catalog contains only explicit Send Copy snapshots (up to 128), not arbitrary
phone files. IDs survive edits within a document session and persisted shared snapshots;
reopening the same file creates a new session identity. Watch stores at most eight copies,
8 MiB per document and 48 MiB for the encoded archive. Legacy data migrates once and stays
recoverable on disk; removing a migrated copy no longer resurrects it on restart.

## Revisions and automated verification

- `038b15c`: all 26 Rust workspace tests, formatting/clippy and source-policy repair
  verified by specialist; independent reviewer reproduced generation/ancestry safety.
- `997354c`: required `scripts/check`, host viewer/source/HTTP replay, all Apple simulator
  family builds passed. iPhone native adapter replay passed in disposable Source Test
  simulator; the user's existing iPhone simulator drawing was left untouched.
- `9fe5de6`: source host replay and iOS/Watch simulator rebuilds passed after asynchronous
  response, cache migration and native status repairs. Watch source runtime replay passed,
  with the URLProtocol transport fixture explicitly skipped on watchOS.
- `15e098b`: final source/callback implementation. Watch build and runtime replay passed,
  including production callback bridges invoked from a utility queue. Native Library
  unavailable-phone error path was then exercised successfully in the running app.
- `4803f10`: final minor empty-state wording and task status. Final device-family build
  and required integration checks passed; results are recorded in the verification logs.

Host: Apple M2 Pro /16 GiB, macOS 27.0.1, Xcode27, Swift6.4, Rust1.99. Runtime:
watchOS27, Watch SE3 40mm; iOS27, isolated iPhone18Pro simulator. GPU measurements were
not run and no latency improvement is claimed. Native automation was exclusive to lead.

Generated logs and screenshots: `artifacts/source-desktop-verification/`. Relevant files:
`integration-check.txt`, `integration-check-final.txt`, `watch-sources-final.txt`,
`apple-simulator-build.txt`, `apple-device-build-final.txt`, `ios-runtime.txt`,
`watch-runtime-final.txt`, `watch-main.png`, `watch-library.png`, `watch-large.png`.
The main screenshot predates callback-only repairs; library/large-text screenshots show
`15e098b`. Fixtures are synthetic and are not evidence of delivered iPhone files.

## Review, failures and repairs

Configured designer defined and reviewed interactions; implementation used isolated
worktrees. A fresh reviewer independently inspected code and reproduced failures/fixes.
Resolved findings: same-content revisions lost metadata advancement, older replies replaced
newer staged content, deferred phone shares read a changed document identity, deleted legacy
copies remigrated, busy library choices silently did nothing, and errors stayed latched.
The designer accepted normal/large-text layout with bounded library names and clear status.

Live inspection uncovered a WatchConnectivity callback executor crash not covered by the
initial adapter replay. Explicit nonisolated/Sendable callbacks now hop to MainActor, and
both the regression replay and formerly crashing native Library action pass. Initial Watch
HTTP replay failed because URLProtocol interception was not honored and tried fixture DNS.
HTTP tests remain mandatory on macOS; watchOS clearly skips that fixture while running shared
Rust/cache/coordinator tests. The launch script now fails unless the replay PASS marker is
present, because simctl can return success after the application reports failure.

## Remaining limits

Physical paired transfer, Digital Crown and VoiceOver speech, real HTTPS server deployment,
authentication/account switching and provisioned independent installation remain unverified.
ARM64 Watch targets only. Unknown or skipped draft ancestry is conservatively a conflict;
there is no general synchronization/merge engine. No automatic cloud upload or continuous
background synchronization. Fetch/list refresh is explicit, with cached-first startup.

At accessibility3, Details truncates visually (full accessible label retained). Scrolling
the primary screen must start outside the drawing because dragging the drawing pans it.
Large documents and UI-thread validation/cache encoding remain unmeasured. Cache deletion
removes downloaded copies, not the original or server file. Release signing/distribution and
renderer selection remain open. Nothing was merged or published.

## Reproduction

```sh
./scripts/check
./scripts/check-apple-viewers
./scripts/build-apple-family simulator
./scripts/build-apple-family device
./scripts/run-watchos <Watch-simulator-UUID> --integration-test
./scripts/run-ios <disposable-iPhone-UUID> --integration-test
```
