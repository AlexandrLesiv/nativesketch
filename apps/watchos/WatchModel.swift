import Darwin
import SwiftUI
import WatchConnectivity

@MainActor final class WatchModel: ObservableObject {
  @Published var navigation = ViewerNavigation()
  @Published var message = "Open Library to choose a shared drawing"
  @Published var version = 0
  @Published var hasError = false
  @Published var catalog: [SourceDescriptor] = []
  @Published var busy = false
  let store: WatchLibraryStore
  let phone = PhoneDrawingSource()
  let coordinator: SourceCoordinator
  init() {
    let root = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
    store = WatchLibraryStore(url: root.appendingPathComponent("WatchLibrary.json"))
    // A build-time Info.plist endpoint is optional. No bundled account credentials.
    let configured = (Bundle.main.object(forInfoDictionaryKey: "NativeSketchServerURL") as? String)
      .flatMap(URL.init(string:))
    coordinator = SourceCoordinator(
      phone: phone, server: ServerDrawingSource(base: configured), store: store)
    coordinator.changed = { [weak self] in self?.version += 1 }
    coordinator.report = { [weak self] text in
      self?.message = text
      self?.hasError = false
    }
    coordinator.failed = { [weak self] text in
      self?.message = text
      self?.hasError = true
    }
    phone.report = { [weak self] text in
      self?.message = text
      self?.hasError = true
    }
    phone.pushed = { [weak self] copy in
      do { try self?.coordinator.pushed(copy) } catch {
        self?.message = error.localizedDescription
        self?.hasError = true
      }
    }
    phone.legacy = { [weak self] data, metadata in
      let hash = SourceDescriptor.hash(data)
      let descriptor = SourceDescriptor(
        namespaceID: "legacy-phone", documentID: hash, revisionID: hash, parentRevisionID: "",
        contentSHA256: hash, serverGeneration: 0, kind: 0, title: metadata.title)
      do { try self?.coordinator.pushed(SourceCopy(descriptor: descriptor, document: data)) } catch
      { self?.message = error.localizedDescription }
    }
    do {
      try store.restore()
      if store.archive.entries.isEmpty {
        let legacy = WatchCopyStore(cache: root.appendingPathComponent("LastCopy.json"))
        try legacy.restore()
        try store.migrate(legacy)
      }
      catalog = store.archive.entries.map { $0.copy.descriptor }
      if store.selected != nil { message = "Cached copy available offline" }
    } catch {
      hasError = true
      message = "Cached library could not be opened. Previous cache retained."
    }
    #if DEBUG
      if ProcessInfo.processInfo.arguments.contains("--integration-test") {
        Task { @MainActor in
          do {
            try ViewerReplay.run()
            try await SourceReplay.run()
            print("APPLE SOURCE REPLAY PASS")
            exit(0)
          } catch {
            print("APPLE SOURCE REPLAY FAIL: \(error)")
            exit(1)
          }
        }
      }
      if ProcessInfo.processInfo.arguments.contains("--preview-sample") {
        do {
          guard let url = Bundle.main.url(forResource: "Sample", withExtension: "nsketch") else {
            throw ViewerError.invalid
          }
          let data = try boundedRead(url, limit: WatchCopyStore.limit)
          let hash = SourceDescriptor.hash(data)
          let descriptor = SourceDescriptor(
            namespaceID: "simulator", documentID: "sample", revisionID: hash, parentRevisionID: "",
            contentSHA256: hash, serverGeneration: 0, kind: 0, title: "Simulator sample drawing")
          try store.accept(
            SourceCopy(descriptor: descriptor, document: data), source: "Simulator fixture",
            select: true)
          catalog = store.archive.entries.map { $0.copy.descriptor }
          version += 1
          message = "Simulator sample · No paired transfer"
        } catch {
          hasError = true
          message = error.localizedDescription
        }
      }
    #endif
  }
  func loadLibrary() {
    guard !busy else { return }
    busy = true
    Task { @MainActor in
      defer { busy = false }
      do {
        let remote = try await coordinator.list()
        let cached = store.archive.entries.map { $0.copy.descriptor }
        catalog =
          cached + remote.filter { candidate in !cached.contains(where: { $0.id == candidate.id }) }
        hasError = false
        message = "Shared drawings. Cached copies open offline."
      } catch {
        catalog = store.archive.entries.map { $0.copy.descriptor }
        hasError = true
        message = error.localizedDescription
      }
    }
  }
  func choose(_ descriptor: SourceDescriptor) {
    coordinator.cancel()
    navigation.fit()
    if store.archive.entries.contains(where: { $0.copy.descriptor.id == descriptor.id }) {
      do {
        try store.select(descriptor.id)
        hasError = false
        message = "Cached copy available offline"
        version += 1
      } catch { message = error.localizedDescription }
    } else {
      refresh(descriptor)
    }
  }
  func refresh(_ descriptor: SourceDescriptor? = nil) {
    guard !busy, let target = descriptor ?? store.selected?.copy.descriptor else { return }
    busy = true
    coordinator.navigating = navigation.zoom != 1 || navigation.pan != .zero
    Task { @MainActor in
      await coordinator.refresh(target)
      busy = false
    }
  }
  func remove(_ item: SourceDescriptor) {
    coordinator.cancel()
    do {
      try store.remove(item.id)
      catalog.removeAll { $0.id == item.id }
      version += 1
      hasError = false
      message = "Cached copy removed. The source is unchanged."
    } catch { message = error.localizedDescription }
  }
  func apply() {
    do {
      try coordinator.apply()
      hasError = false
      message = "Update applied"
      navigation.fit()
      version += 1
    } catch { message = error.localizedDescription }
  }
}
