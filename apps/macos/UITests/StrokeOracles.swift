import AppKit
import XCTest

// Independent read-only observations; never feed document bytes or commands into the app.
func verifyMouseStroke(_ data: Data, canvasSize: CGSize) throws {
  func integer(_ offset: Int) throws -> UInt64 {
    guard offset >= 0, data.count >= offset + 8 else {
      XCTFail("Truncated document at byte \(offset)")
      throw CocoaError(.fileReadCorruptFile)
    }
    return data[offset..<(offset + 8)].enumerated().reduce(0) {
      $0 | (UInt64($1.element) << ($1.offset * 8))
    }
  }
  func number(_ offset: Int) throws -> Double { Double(bitPattern: try integer(offset)) }
  XCTAssertEqual(String(data: data.prefix(8), encoding: .utf8), "NSKETCH1")
  XCTAssertEqual(try integer(8), 1)
  XCTAssertEqual(try integer(24), 0, "XCTest pointer input is mouse, not physical pen")
  XCTAssertEqual(try number(40), 1, accuracy: 0.01)
  XCTAssertEqual(try number(48), 0, accuracy: 0.01)
  XCTAssertEqual(try number(56), 0, accuracy: 0.01)
  XCTAssertEqual(try number(64), 1, accuracy: 0.01)
  XCTAssertEqual(try number(72), 12)
  XCTAssertEqual(try number(80), 1, "New strokes must have identity scale")
  XCTAssertEqual(try number(88), 0, "New strokes must not translate horizontally")
  XCTAssertEqual(try number(96), 0, "New strokes must not translate vertically")
  let count = try integer(104)
  XCTAssertGreaterThanOrEqual(count, 2)
  guard count <= UInt64(max(0, data.count - 120) / 48) else {
    XCTFail("Invalid sample count")
    throw CocoaError(.fileReadCorruptFile)
  }
  var lastID: UInt64 = 0
  var lastTime = -Double.infinity
  var points: [CGPoint] = []
  for sample in 0..<Int(count) {
    let offset = 112 + sample * 48
    let id = try integer(offset)
    let time = try number(offset + 24)
    XCTAssertGreaterThan(id, lastID)
    XCTAssertGreaterThanOrEqual(time, lastTime)
    XCTAssertEqual(try integer(offset + 32), 0, "Mouse has no pressure capability")
    points.append(CGPoint(x: try number(offset + 8), y: try number(offset + 16)))
    lastID = id
    lastTime = time
  }
  XCTAssertTrue(
    StrokeExpectation.endpointsMatch(points, canvasSize: canvasSize),
    "Saved endpoints must match the intended logical canvas trajectory")
  let first = try XCTUnwrap(points.first)
  let last = try XCTUnwrap(points.last)
  XCTAssertGreaterThan(last.x - first.x, 100)
  XCTAssertGreaterThan(last.y - first.y, 40)
  for point in points {
    XCTAssertTrue(point.x.isFinite && point.y.isFinite)
    let fraction = (point.x - first.x) / (last.x - first.x)
    XCTAssertEqual(point.y, first.y + fraction * (last.y - first.y), accuracy: 8)
  }
}

func redInkPixels(_ screenshot: XCUIScreenshot) throws -> [Int] {
  let bitmap = try XCTUnwrap(NSBitmapImageRep(data: screenshot.pngRepresentation))
  var counts: [Int] = []
  // Three interior patches exclude cursor endpoints, window controls and empty hints.
  for position in [0.35, 0.5, 0.65] {
    var matches = 0
    let x = Int(Double(bitmap.pixelsWide) * position)
    let y = Int(Double(bitmap.pixelsHigh) * (0.4 + (position - 0.3) * 0.5))
    for column in (x - 4)...(x + 4) {
      for row in (y - 4)...(y + 4) {
        let color = try XCTUnwrap(bitmap.colorAt(x: column, y: row)?.usingColorSpace(.deviceRGB))
        if color.redComponent > 0.7 && color.greenComponent < 0.3 && color.blueComponent < 0.3 {
          matches += 1
        }
      }
    }
    counts.append(matches)
  }
  return counts
}
