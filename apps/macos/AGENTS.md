# macOS adapter

Use Swift and real AppKit controls for native interactions. SwiftUI is acceptable only
where it preserves the requested native behavior. Keep custom graphics in the canvas.
Native UI and lifecycle stay on the appropriate main-thread boundaries. Avoid publishing
each pen sample as broad UI state. Read docs/design/principles.md for visible changes.
Run the app and exercise the changed interaction; a successful build is not UI verification.

