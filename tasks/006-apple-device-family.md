# 006: Apple device family

Status: review. Authorized by owner on 2026-10-14, based on platform branch `30d472b`.

## Outcome

Keep AppKit Mac and UIKit iPhone/iPad editing; add a native visionOS 2D editor and
native tvOS/watchOS viewers. Shared Rust validates documents and produces geometry.
No permanent renderer choice, release distribution or main-branch merge.

## Agreed contracts

- Existing C ABI 2 remains unchanged. Viewer wrappers privately own Rust handles and
  expose only validated import, read-only state and path copies. No edit/history/save UI.
- Vision uses UIKit controls, explicit Pen, Select/Pan default, indirect pinch navigation,
  native Files access and existing transactional editing/dirty guards.
- TV opens an explicit HTTPS document URL, bounded at 64 MiB, or a bundled sample.
  Download cancellation, stale completion and invalid input preserve the prior document.
  Native remote-focusable Fit/Zoom/Pan controls affect only viewport state.
- Watch receives an explicit iPhone document copy through WatchConnectivity. Maximum
  transfer size is 8 MiB; metadata is protocolVersion=1, sentAt (Unix seconds), title,
  contentType="dev.nativesketch.document". Preserve last valid copy on invalid/stale input.
  Move the temporary incoming file before the receive delegate returns. Sending never
  marks the source editor saved. Queue completion must not imply display confirmation.
- Watch app identifier is dev.nativesketch.ios.watch; companion is dev.nativesketch.ios.
  ARM64 watch targets use stable Rust. Older arm64_32 watch hardware is outside this build.
- TV cached files may be purged by the OS. Viewer copies are not live synchronization.

## Acceptance and evidence

Build simulator/device variants with explicit SDK and Rust triples. Verify read-only
byte identity, corrupt/oversized input preservation, stale delivery, navigation and
source dirty-state retention. Run available simulator adapter replays exclusively on
this host; inspect native interfaces where tooling permits. Keep physical Vision input,
Siri Remote, WatchConnectivity pairing and accessibility speech distinct from replay.
Designer review and fresh independent review precede owner merge review.

Evidence: [Apple device family verification](../docs/evidence/006-apple-device-family.md).
Final code revision `070412b`; implementation and review complete, awaiting owner merge approval.
