import CSketchFFI
import UIKit
import UniformTypeIdentifiers

@MainActor final class DrawingView: UIView {
  unowned let owner: DrawingController
  var zoom = 1.0
  var offset = CGPoint(x: 32, y: 32)
  var mode = 0  // idle, ink, resize. Navigation is view state, never document state.
  var sampleID: UInt64 = 0
  var lastTimestamp = -Double.infinity
  var anchor = CGPoint.zero
  var initialVector = CGPoint.zero
  var ownerTouch: UITouch?
  var contacts = Set<UITouch>()
  var navigationTouches: [UITouch] = []
  var navigating = false
  var previousNavigation: (CGPoint, Double)?
  var oldSize = CGSize.zero
  init(owner: DrawingController) {
    self.owner = owner
    super.init(frame: .zero)
    backgroundColor = .white
    #if os(visionOS)
      let drag = UIPanGestureRecognizer(target: self, action: #selector(spatialDrag))
      addGestureRecognizer(drag)
      let tap = UITapGestureRecognizer(target: self, action: #selector(spatialTap))
      tap.require(toFail: drag)
      addGestureRecognizer(tap)
    #else
      isMultipleTouchEnabled = true
    #endif
    isAccessibilityElement = true
    accessibilityLabel = "Drawing canvas"
    accessibilityHint =
      "Use Select and Next Stroke to select without drawing. Two fingers pan and zoom."
    accessibilityCustomActions = [
      UIAccessibilityCustomAction(
        name: "Next stroke", target: owner, selector: #selector(DrawingController.nextStroke)),
      UIAccessibilityCustomAction(
        name: "Previous stroke", target: owner,
        selector: #selector(DrawingController.previousStroke)),
    ]
  }
  required init?(coder: NSCoder) { fatalError() }
  override var canBecomeFirstResponder: Bool { true }
  var editor: MobileEditor { owner.editor }
  // Preserve fitted scales below 10%; navigation must never jump up to a
  // desktop-sized floor after fitting a very large, valid Rust document.
  func navigationZoom(_ proposed: Double) -> Double {
    guard proposed.isFinite, proposed > 0 else { return zoom }
    return min(16, max(min(zoom, 1e-15), proposed))
  }
  func document(_ p: CGPoint) -> CGPoint {
    CGPoint(x: (p.x - offset.x) / zoom, y: (p.y - offset.y) / zoom)
  }
  func screen(_ p: CGPoint) -> CGPoint {
    CGPoint(x: p.x * zoom + offset.x, y: p.y * zoom + offset.y)
  }
  func cancel() {
    if editor.handle != nil { _ = editor.check(sketch_cancel(editor.handle)) }
    mode = 0
    ownerTouch = nil
    setNeedsDisplay()
  }
  func cancelAll() {
    cancel()
    contacts.removeAll()
    navigationTouches.removeAll()
    navigating = false
    previousNavigation = nil
    owner.refresh()
  }
  override func layoutSubviews() {
    super.layoutSubviews()
    if oldSize != .zero && oldSize != bounds.size {
      cancelAll()
      offset.x += (bounds.width - oldSize.width) / 2
      offset.y += (bounds.height - oldSize.height) / 2
    }
    oldSize = bounds.size
    contentScaleFactor = traitCollection.displayScale
    setNeedsDisplay()
  }
  override func didMoveToWindow() {
    super.didMoveToWindow()
    if window == nil { cancelAll() }
    contentScaleFactor = traitCollection.displayScale
  }
  #if os(visionOS)
    var panOrigin = CGPoint.zero
  #endif
  override func draw(_ rect: CGRect) { paint() }
  #if !os(visionOS)
    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
      inputBegan(touches, with: event)
    }
    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
      inputMoved(touches, with: event)
    }
    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
      inputEnded(touches, with: event)
    }
    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) { cancelAll() }
  #endif

}
