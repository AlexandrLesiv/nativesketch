#if DEBUG
  import Foundation
  @MainActor final class StubDrawingSource: DrawingSource {
    var copies: [SourceCopy]
    var failure = false
    var delay: Duration = .zero
    var fetches = 0
    init(_ copies: [SourceCopy]) { self.copies = copies }
    func list() async throws -> [SourceDescriptor] {
      if failure { throw SourceFailure.unavailable }
      return copies.map(\.descriptor)
    }
    func fetch(_ descriptor: SourceDescriptor) async throws -> SourceCopy {
      fetches += 1
      if delay != .zero { try await Task.sleep(for: delay) }
      if failure { throw SourceFailure.unavailable }
      guard let copy = copies.first(where: { $0.descriptor.id == descriptor.id }) else {
        throw SourceFailure.unavailable
      }
      return copy
    }
  }
#endif
