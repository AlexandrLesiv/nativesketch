# Apple device capability agreement

Designed with ui_ux_designer before implementation, 2026-10-14.

| Family | Mode | Native controls |
| --- | --- | --- |
| Mac | Edit | AppKit |
| iPhone / iPad | Edit | UIKit |
| Vision Pro | Edit in a resizable 2D window | UIKit; explicit Pen, Select/Pan, indirect pinch navigation |
| Apple TV | View only | UIKit remote-focusable Open URL, Sample, Fit, Zoom and Pan |
| Apple Watch | View only | SwiftUI, Fit, Digital Crown zoom and drag pan |

Apple system controls supply current appearance, including Liquid Glass where the OS
provides it. Do not fabricate glass over drawing content. Vision is a flat document
editor, not an immersive spatial drawing tool. Start in Select/Pan to prevent accidental
ink from ordinary spatial selection. Do not infer pressure from indirect pinch strength.

TV opens an explicit HTTPS document link or bundled sample. A native text-entry flow
and remote focus reach every command. Show name and View Only. Load/cancel/failure must
retain the last valid drawing; stale requests must not replace a newer result. Fit shows
all content. Cache may be purged by tvOS: expose empty/reload recovery honestly.

Watch receives a document copy explicitly sent from iPhone. Show name, View Only and
received time; empty instructions explain sending a copy from the phone. Read-only Rust
import/state/render calls validate and display portable document bytes. View navigation
never changes bytes, history or source dirty state. Fit shows full drawing; Crown zoom
is bounded and anchored to the viewport center. Native labels explain capability/status.

The phone says queued when WatchConnectivity accepts the file; it must not claim the
watch displayed it without acknowledgment. Move the incoming temporary URL before the
receive callback returns, validate bounded bytes, then atomically replace the cache and
presentation. Reject stale or invalid copies without losing the last good drawing.
These are independent copies, not live synchronization or durable cloud storage.

Acceptance includes empty/loaded/loading/cancelled/error states; narrow/large layouts;
native focus and labels; corrupt, oversized and stale inputs; navigation identity; and
retained source unsaved state. Build, adapter replay, simulator UI and physical device
acceptance are separate evidence. ARM64 Watch builds do not certify older arm64_32 devices.

Primary design references: [Apple indirect input](https://developer.apple.com/documentation/bundleresources/information-property-list/uiapplicationsupportsindirectinputevents),
[WatchConnectivity temporary files](https://developer.apple.com/documentation/watchconnectivity/wcsessionfile/fileurl),
and [Apple TV storage](https://developer.apple.com/videos/play/wwdc2023/10256/).
