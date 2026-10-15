import AppKit
import CSketchSession

@MainActor final class CanvasView: NSView {
  unowned let owner: EditorWindowController
  var zoom = 1.0
  var offset = CGPoint(x: 80, y: 80)
  lazy var input = AppKitInputAdapter(canvas: self)
  let renderDiagnostics = RenderDiagnostics()
  lazy var rendering = CanvasRendering(canvas: self)
  init(owner: EditorWindowController) {
    self.owner = owner
    super.init(frame: .zero)
    setAccessibilityElement(true)
    setAccessibilityRole(.group)
    NativeAccessibility.label(self, id: "sketch.canvas", text: "Drawing canvas")
    setAccessibilityHelp(
      "P for Pen, V for Select. Space drag to pan. Select strokes using the Edit menu, then use Scale percent to resize."
    )
  }
  required init?(coder: NSCoder) { fatalError() }
  override var isFlipped: Bool { true }
  override var acceptsFirstResponder: Bool { true }
  override func becomeFirstResponder() -> Bool {
    let accepted = super.becomeFirstResponder()
    if accepted { needsDisplay = true }
    return accepted
  }
  var editor: RustApplicationSession { owner.editor }
  override func accessibilityValue() -> Any? {
    let value = super.accessibilityValue()
    return owner.uiObservation?.value(appendingTo: value) ?? value
  }
  func document(_ p: CGPoint) -> CGPoint {
    CGPoint(x: (p.x - offset.x) / zoom, y: (p.y - offset.y) / zoom)
  }
  func screen(_ p: CGPoint) -> CGPoint {
    CGPoint(x: p.x * zoom + offset.x, y: p.y * zoom + offset.y)
  }
  func point(_ event: NSEvent) -> CGPoint { convert(event.locationInWindow, from: nil) }
  func cancel() {
    if editor.projection.interaction != 0 { editor.send(.cancel) }
    input.gesture = .idle
    input.space = false
    owner.refresh()
  }
  override func cancelOperation(_ sender: Any?) { escape() }
  func escape() {
    let idle = input.gesture == .idle
    cancel()
    if idle {
      editor.send(.clearSelection)
      owner.refresh()
    }
  }
  override func viewDidChangeBackingProperties() {
    super.viewDidChangeBackingProperties()
    needsDisplay = true
  }
  override func setFrameSize(_ size: NSSize) {
    let old = frame.size
    if old != .zero && old != size {
      cancel()
      offset.x += (size.width - old.width) / 2
      offset.y += (size.height - old.height) / 2
    }
    super.setFrameSize(size)
  }
  func corners(_ b: SketchBoundsV1) -> [CGPoint] {
    [
      CGPoint(x: b.x, y: b.y), CGPoint(x: b.x + b.width, y: b.y),
      CGPoint(x: b.x + b.width, y: b.y + b.height), CGPoint(x: b.x, y: b.y + b.height),
    ]
  }
  override var wantsUpdateLayer: Bool { true }
  override func updateLayer() { rendering.invalidate() }
  override func viewDidMoveToWindow() {
    super.viewDidMoveToWindow()
    rendering.attach()
  }
  override func viewDidChangeEffectiveAppearance() {
    super.viewDidChangeEffectiveAppearance()
    needsDisplay = true
  }
  override func mouseDown(with event: NSEvent) {
    owner.focusRestoration.discard()
    input.mouseDown(with: event)
  }
  override func mouseDragged(with event: NSEvent) { input.mouseDragged(with: event) }
  override func mouseUp(with event: NSEvent) { input.mouseUp(with: event) }
  override func scrollWheel(with event: NSEvent) {
    guard input.gesture == .idle else { return }
    offset.x -= event.scrollingDeltaX
    offset.y -= event.scrollingDeltaY
    needsDisplay = true
  }
  func resetViewport() {
    input.gesture = .idle
    input.space = false
    zoom = 1
    offset = CGPoint(x: 80, y: 80)
  }
  func setZoom(_ value: Double, at point: CGPoint? = nil) {
    guard input.gesture == .idle else { return }
    let p = point ?? CGPoint(x: bounds.midX, y: bounds.midY)
    let d = document(p)
    zoom = min(16, max(0.1, value))
    offset = CGPoint(x: p.x - d.x * zoom, y: p.y - d.y * zoom)
    owner.refresh()
  }
  override func magnify(with event: NSEvent) {
    setZoom(zoom * (1 + event.magnification), at: point(event))
  }
  override func keyDown(with event: NSEvent) {
    owner.focusRestoration.discard()
    switch event.charactersIgnoringModifiers?.lowercased() {
    case " ": if input.gesture == .idle { input.space = true }
    case "p":
      owner.actions.penTool(nil)
    case "v":
      owner.actions.selectTool(nil)
    case "\u{7f}", "\u{f728}": owner.actions.deleteStroke(nil)
    case "\u{1b}": escape()
    default: super.keyDown(with: event)
    }
  }
  override func keyUp(with event: NSEvent) {
    if event.charactersIgnoringModifiers == " " {
      input.space = false
    } else {
      super.keyUp(with: event)
    }
  }
  override func resignFirstResponder() -> Bool {
    cancel()
    return super.resignFirstResponder()
  }
}
