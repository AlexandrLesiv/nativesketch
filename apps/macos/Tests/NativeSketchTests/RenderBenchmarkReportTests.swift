import Foundation
import Testing

@testable import NativeSketch

struct RenderBenchmarkReportTests {
  @Test func repeatedOutputCannotReplaceFailedEvidence() throws {
    let directory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    defer { try? FileManager.default.removeItem(at: directory) }
    let output = directory.appendingPathComponent("attempt.json")
    var failed = RenderBenchmarkReport()
    failed.reason = "external-window-overlaps-canvas"
    try failed.write(to: output)
    let original = try Data(contentsOf: output)
    var retry = RenderBenchmarkReport()
    retry.valid = true
    #expect(throws: (any Error).self) { try retry.write(to: output) }
    #expect(try Data(contentsOf: output) == original)
  }
}
