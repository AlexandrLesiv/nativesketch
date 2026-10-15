import Foundation

/// The journey uses File > New after layout: the current native contract resets
/// zoom to 1 and document origin to (80, 80). AX canvas dimensions are logical points;
/// Canvas is flipped, so XCUI's top-left-relative y needs no display-pixel conversion.
enum StrokeExpectation {
  static func containsInkInEveryPatch(_ counts: [Int]) -> Bool {
    counts.count == 3 && counts.allSatisfy { $0 > 20 }
  }

  static func endpointsMatch(_ points: [CGPoint], canvasSize: CGSize) -> Bool {
    guard let first = points.first, let last = points.last,
      canvasSize.width > 0, canvasSize.height > 0
    else { return false }
    let expectedFirst = CGPoint(x: canvasSize.width * 0.3 - 80, y: canvasSize.height * 0.4 - 80)
    let expectedLast = CGPoint(x: canvasSize.width * 0.7 - 80, y: canvasSize.height * 0.6 - 80)
    // Two logical points allow OS pointer-coordinate rounding, including Retina,
    // without accepting meaningful translation, scale or axis errors.
    return zip([first, last], [expectedFirst, expectedLast]).allSatisfy { actual, expected in
      actual.x.isFinite && actual.y.isFinite
        && abs(actual.x - expected.x) <= 2 && abs(actual.y - expected.y) <= 2
    }
  }
}
