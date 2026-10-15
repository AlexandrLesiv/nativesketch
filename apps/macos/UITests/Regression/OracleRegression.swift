import Foundation

@main
struct OracleRegression {
  static func main() {
    precondition(!StrokeExpectation.containsInkInEveryPatch([81, 0, 0]))
    precondition(!StrokeExpectation.containsInkInEveryPatch([21, 20, 21]))
    precondition(!StrokeExpectation.containsInkInEveryPatch([81, 81]))
    precondition(StrokeExpectation.containsInkInEveryPatch([21, 21, 21]))
    let canvas = CGSize(width: 1000, height: 700)
    let intended = [CGPoint(x: 220, y: 200), CGPoint(x: 620, y: 340)]
    precondition(StrokeExpectation.endpointsMatch(intended, canvasSize: canvas))
    let rounded = [CGPoint(x: 221, y: 199), CGPoint(x: 619, y: 341)]
    precondition(StrokeExpectation.endpointsMatch(rounded, canvasSize: canvas))
    let translated = intended.map { CGPoint(x: $0.x + 20, y: $0.y + 20) }
    precondition(!StrokeExpectation.endpointsMatch(translated, canvasSize: canvas))
    let scaled = intended.map { CGPoint(x: $0.x * 2, y: $0.y * 2) }
    precondition(!StrokeExpectation.endpointsMatch(scaled, canvasSize: canvas))
    precondition(!StrokeExpectation.endpointsMatch([], canvasSize: canvas))
    let invalid = [CGPoint(x: CGFloat.nan, y: 200), intended[1]]
    precondition(!StrokeExpectation.endpointsMatch(invalid, canvasSize: canvas))
    print("PASS: independent pixel-patch and endpoint oracle regressions (no native UI)")
  }
}
