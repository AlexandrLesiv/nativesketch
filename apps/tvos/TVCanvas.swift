import Darwin
import Foundation
import UIKit

@MainActor final class TVCanvas: UIView {
  var document: ReadOnlyDocument
  var navigation = ViewerNavigation() { didSet { setNeedsDisplay() } }
  init(_ document: ReadOnlyDocument) {
    self.document = document
    super.init(frame: .zero)
    backgroundColor = .white
    isAccessibilityElement = true
    accessibilityLabel = "Drawing preview, view only"
  }
  required init?(coder: NSCoder) { fatalError() }
  override func draw(_ rect: CGRect) {
    guard let context = UIGraphicsGetCurrentContext() else { return }
    context.concatenate(navigation.transform(bounds: document.bounds, size: bounds.size))
    for (path, color) in document.paths {
      context.addPath(path)
      context.setFillColor(color)
      context.fillPath()
    }
  }
}
