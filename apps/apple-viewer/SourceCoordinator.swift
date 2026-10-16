import CSketchFFI
import Foundation

// Single owner of request generations. Late responses cannot update another selection.
@MainActor final class SourceCoordinator {
  let phone: any DrawingSource
  let server: any DrawingSource
  let store: WatchLibraryStore
  private var generation = 0
  var navigating = false
  private(set) var pending: (SourceCopy, String)?
  var changed: () -> Void = {}
  var report: (String) -> Void = { _ in }
  var failed: (String) -> Void = { _ in }
  init(phone: any DrawingSource, server: any DrawingSource, store: WatchLibraryStore) {
    self.phone = phone
    self.server = server
    self.store = store
  }
  func cancel() {
    generation += 1
    pending = nil
  }
  func list() async throws -> [SourceDescriptor] {
    do { return try await phone.list() } catch { return try await server.list() }
  }
  func refresh(_ descriptor: SourceDescriptor) async {
    generation += 1
    let request = generation
    do {
      let copy: SourceCopy
      let source: String
      do {
        copy = try await phone.fetch(descriptor)
        source = "iPhone"
      } catch {
        guard request == generation else { return }
        guard !descriptor.phoneOnly else { throw SourceFailure.phoneOnly }
        // Fetch catalog again to discover a newer server revision of exactly this identity.
        let catalog = try await server.list()
        guard let latest = catalog.first(where: { $0.id == descriptor.id }) else {
          throw SourceFailure.unavailable
        }
        copy = try await server.fetch(latest)
        source = "Server"
      }
      guard request == generation else { return }
      guard copy.descriptor.id == descriptor.id else { throw SourceFailure.rejected }
      try copy.validate()
      let decision = try compareSource(descriptor, copy.descriptor)
      guard decision == 0 || decision == 1 else { throw SourceFailure.rejected }
      try checkPending(copy)
      if navigating, store.selected?.copy.descriptor.id == descriptor.id,
        store.selected?.copy.descriptor.contentSHA256 != copy.descriptor.contentSHA256
      {
        try stage(copy, source: source)
        report("New version available. Use Apply Update in Details.")
        changed()
      } else {
        try store.accept(copy, source: source, select: true)
        pending = nil
        report("Updated from " + source)
        changed()
      }
    } catch {
      if request == generation {
        failed(error.localizedDescription)
        changed()
      }
    }
  }
  private func checkPending(_ copy: SourceCopy) throws {
    if let pending, pending.0.descriptor.id == copy.descriptor.id {
      let decision = try compareSource(pending.0.descriptor, copy.descriptor)
      guard decision == 0 || decision == 1 else { throw SourceFailure.rejected }
    }
  }
  private func stage(_ copy: SourceCopy, source: String) throws {
    try checkPending(copy)
    let candidate = ReadOnlyDocument()
    try candidate.importCopy(copy.document, limit: WatchCopyStore.limit)
    pending = (copy, source)
  }
  func apply() throws {
    guard let (copy, source) = pending, copy.descriptor.id == store.selected?.copy.descriptor.id
    else {
      pending = nil
      return
    }
    try store.accept(copy, source: source, select: true)
    pending = nil
    navigating = false
    changed()
  }
  func pushed(_ copy: SourceCopy) throws {
    try copy.validate()
    try checkPending(copy)
    // A push adds to the library; it never hijacks an existing selection.
    if copy.descriptor.id == store.selected?.copy.descriptor.id, navigating,
      store.selected?.copy.descriptor.contentSHA256 != copy.descriptor.contentSHA256
    {
      if let current = store.selected {
        let decision = try compareSource(current.copy.descriptor, copy.descriptor)
        guard decision == 0 || decision == 1 else { throw SourceFailure.rejected }
      }
      try stage(copy, source: "iPhone")
      report("New version available. Use Apply Update in Details.")
    } else {
      try store.accept(copy, source: "iPhone", select: store.selected == nil)
      if pending?.0.descriptor.id == copy.descriptor.id { pending = nil }
    }
    changed()
  }
}
