# macOS adapter

Use Swift and real AppKit controls for native interactions. SwiftUI is acceptable only
where it preserves the requested native behavior. Keep custom graphics in the canvas.
Native UI and lifecycle stay on the appropriate main-thread boundaries. Avoid publishing
each pen sample as broad UI state. Read docs/design/principles.md for visible changes.
Run the app and exercise the changed interaction; a successful build is not UI verification.
Use presenters and typed bridge calls: canvas input must not read toolbar widgets.
Keep field editing/focus local and validated tool state in Rust. Preserve text undo routing.
Read docs/architecture/contracts.md and docs/quality/platforms.md. Add XCTest/XCUIAutomation
coverage for changed user journeys, plus bridge/replay checks for precise sample semantics.
Record real versus simulated input; physical pen-to-screen measurements need a camera.
Keep new/extracted source and test files at most 200 physical lines. Expose accessibility
identifiers and bounded diagnostic spans; validate resize/display/focus cancellation.

