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
    let configured = (Bundle.main.object(forInfoDictionaryKey: "NativeSketchServerURL") as? String).flatMap(URL.init(string:))
    coordinator = SourceCoordinator(phone: phone, server: ServerDrawingSource(base: configured), store: store)
    coordinator.changed = { [weak self] in self?.version += 1 }
    coordinator.report = { [weak self] text in self?.message = text; self?.hasError = false }
    coordinator.failed = { [weak self] text in self?.message = text; self?.hasError = true }
    phone.report = { [weak self] text in self?.message = text; self?.hasError = true }
    phone.pushed = { [weak self] copy in
      do { try self?.coordinator.pushed(copy) } catch { self?.message = error.localizedDescription; self?.hasError = true }
    }
    phone.legacy = { [weak self] data, metadata in
      let hash = SourceDescriptor.hash(data)
      let descriptor = SourceDescriptor(namespaceID: "legacy-phone", documentID: hash, revisionID: hash, parentRevisionID: "", contentSHA256: hash, serverGeneration: 0, kind: 0, title: metadata.title)
      do { try self?.coordinator.pushed(SourceCopy(descriptor: descriptor, document: data)) } catch { self?.message = error.localizedDescription }
    }
    do {
      try store.restore()
      if store.archive.entries.isEmpty {
        let legacy = WatchCopyStore(cache: root.appendingPathComponent("LastCopy.json")); try legacy.restore(); try store.migrate(legacy)
      }
      catalog = store.archive.entries.map { $0.copy.descriptor }
      if store.selected != nil { message = "Cached copy available offline" }
    } catch { hasError = true; message = "Cached library could not be opened. Previous cache retained." }
    #if DEBUG
    if ProcessInfo.processInfo.arguments.contains("--integration-test") {
      Task { @MainActor in
        do { try ViewerReplay.run(); try await SourceReplay.run(); print("APPLE SOURCE REPLAY PASS"); exit(0) }
        catch { print("APPLE SOURCE REPLAY FAIL: \(error)"); exit(1) }
      }
    }
    if ProcessInfo.processInfo.arguments.contains("--preview-sample") {
      do {
        guard let url = Bundle.main.url(forResource: "Sample", withExtension: "nsketch") else { throw ViewerError.invalid }
        let data = try boundedRead(url, limit: WatchCopyStore.limit); let hash = SourceDescriptor.hash(data)
        let descriptor = SourceDescriptor(namespaceID: "simulator", documentID: "sample", revisionID: hash, parentRevisionID: "", contentSHA256: hash, serverGeneration: 0, kind: 0, title: "Simulator sample drawing")
        try store.accept(SourceCopy(descriptor: descriptor, document: data), source: "Simulator fixture", select: true)
        catalog = store.archive.entries.map { $0.copy.descriptor }; version += 1; message = "Simulator sample · No paired transfer"
      } catch { hasError = true; message = error.localizedDescription }
    }
    #endif
  }
  func loadLibrary() {
    guard !busy else { return }; busy = true
    Task { @MainActor in
      defer { busy = false }
      do {
        let remote = try await coordinator.list()
        let cached = store.archive.entries.map { $0.copy.descriptor }
        catalog = cached + remote.filter { candidate in !cached.contains(where: { $0.id == candidate.id }) }
        hasError = false; message = "Shared drawings. Cached copies open offline."
      } catch { catalog = store.archive.entries.map { $0.copy.descriptor }; hasError = true; message = error.localizedDescription }
    }
  }
  func choose(_ descriptor: SourceDescriptor) {
    coordinator.cancel(); navigation.fit()
    if store.archive.entries.contains(where: { $0.copy.descriptor.id == descriptor.id }) {
      do { try store.select(descriptor.id); hasError = false; message = "Cached copy available offline"; version += 1 } catch { message = error.localizedDescription }
    } else { refresh(descriptor) }
  }
  func refresh(_ descriptor: SourceDescriptor? = nil) {
    guard !busy, let target = descriptor ?? store.selected?.copy.descriptor else { return }
    busy = true
    coordinator.navigating = navigation.zoom != 1 || navigation.pan != .zero
    Task { @MainActor in await coordinator.refresh(target); busy = false }
  }
  func remove(_ item: SourceDescriptor) {
    coordinator.cancel()
    do { try store.remove(item.id); catalog.removeAll { $0.id == item.id }; version += 1; hasError = false; message = "Cached copy removed. The source is unchanged." } catch { message = error.localizedDescription }
  }
  func apply() { do { try coordinator.apply(); hasError = false; message = "Update applied"; navigation.fit(); version += 1 } catch { message = error.localizedDescription } }
}
struct WatchViewer: View {
  @StateObject private var model = WatchModel()
  @State private var crown = 1.0
  @State private var startPan: CGSize?
  @State private var showingDetails = false
  @Environment(\.dynamicTypeSize) private var dynamicTypeSize
  private var detailsSummary: String {
    let received = (model.store.selected?.receivedAt).map { "Received " + Date(timeIntervalSince1970: $0).formatted(date: .abbreviated, time: .shortened) } ?? ""
    return [model.store.selected?.copy.descriptor.title ?? "", received, model.message].filter { !$0.isEmpty }.joined(separator: ". ")
  }
  var body: some View {
    Group {
      if dynamicTypeSize.isAccessibilitySize {
        // Large text can scroll to every native control without shrinking the canvas.
        ScrollView { primary.frame(minHeight: 280) }
      } else { primary }
    }
    .sheet(isPresented: $showingDetails) { NavigationStack { details } }
  }
  private var primary: some View {
    VStack(spacing: 4) {
      Text("View Only").font(.caption).foregroundStyle(.secondary)
      Canvas { context, size in
        context.fill(Path(CGRect(origin: .zero, size: size)), with: .color(.white))
        context.concatenate(model.navigation.transform(bounds: model.store.document.bounds, size: size))
        for (path, color) in model.store.document.paths { context.fill(Path(path), with: .color(Color(cgColor: color))) }
      }
      .frame(minHeight: 64, maxHeight: .infinity)
      .layoutPriority(1)
      .overlay {
        if model.store.selected == nil {
          Text("Open Details → Library, or send a copy from iPhone").font(.caption2).foregroundStyle(.black)
            .multilineTextAlignment(.center).padding(6).allowsHitTesting(false)
        }
      }
      .accessibilityLabel("Drawing preview, view only")
      .focusable()
      .digitalCrownRotation($crown, from: 0.25, through: 16, by: 0.1, sensitivity: .low, isContinuous: false)
      .onChange(of: model.store.archive.selected) { _, _ in crown = 1; startPan = nil }
      .onChange(of: model.navigation.zoom) { _, value in crown = value }
      .onChange(of: crown) { _, value in model.navigation.zoom = value; model.coordinator.navigating = value != 1 || model.navigation.pan != .zero }
      .gesture(DragGesture().onChanged { value in
        model.coordinator.navigating = true
        if startPan == nil { startPan = model.navigation.pan }
        model.navigation.pan = CGSize(width: (startPan?.width ?? 0) + value.translation.width, height: (startPan?.height ?? 0) + value.translation.height)
      }.onEnded { _ in startPan = nil })
      if let metadata = model.store.selected?.copy.descriptor, !dynamicTypeSize.isAccessibilitySize {
        ViewThatFits(in: .vertical) {
          Text(metadata.title).font(.caption2).lineLimit(1).truncationMode(.tail)
            .accessibilityLabel("Document: " + metadata.title)
          EmptyView()
        }
      }
      if model.hasError { Text("Copy error").font(.caption2).foregroundStyle(.red).accessibilityHint("Open Details for the full error") }
      HStack {
        Button("Fit") { crown = 1; model.navigation.fit(); model.coordinator.navigating = false }
          .disabled(model.store.document.paths.isEmpty)
        Button("Details") { showingDetails = true }
          .accessibilityValue(detailsSummary)
      }
    }.padding(4)
  }
  private var library: some View {
    List {
      Text("Shared drawings").font(.headline)
      if model.catalog.isEmpty { Text("No cached drawings. Share a copy from iPhone, or configure server access.") }
      ForEach(model.catalog) { item in
        Button { model.choose(item); showingDetails = false } label: {
          VStack(alignment: .leading) { Text(item.title).lineLimit(2).truncationMode(.tail).accessibilityLabel(item.title); Text(item.phoneOnly ? "iPhone copy" : "Synced drawing").font(.caption2) }
        }
        .disabled(model.busy)
        .swipeActions { if model.store.archive.entries.contains(where: { $0.copy.descriptor.id == item.id }) { Button("Remove Cache", role: .destructive) { model.remove(item) } } }
      }
      Button(model.busy ? "Refreshing…" : "Refresh Library") { model.loadLibrary() }.disabled(model.busy)
      Text(model.message).font(.caption2)
      Button("Done") { showingDetails = false }
    }
  }
  private var details: some View {
      ScrollView {
        VStack(alignment: .leading, spacing: 12) {
          Text("View Only").font(.headline)
          NavigationLink("Library") { library.onAppear { model.loadLibrary() } }
          Button(model.busy ? "Refreshing…" : "Refresh") { model.refresh() }.disabled(model.busy || model.store.selected == nil)
          if model.coordinator.pending != nil { Button("Apply Update") { model.apply() } }
          if let selected = model.store.selected { Text("Source: " + selected.source) }
          if let metadata = model.store.selected?.copy.descriptor {
            Text(metadata.title).fixedSize(horizontal: false, vertical: true)
            Text("Received " + Date(timeIntervalSince1970: model.store.selected?.receivedAt ?? 0).formatted(date: .complete, time: .shortened))
          }
          if !model.message.isEmpty { Text(model.message).fixedSize(horizontal: false, vertical: true) }
          Text("Turn the Digital Crown to zoom. Drag to pan. Fit shows the entire drawing.")
          Button("Done") { showingDetails = false }
        }.padding()
      }
  }
}
@main struct NativeSketchWatch: App {
  var body: some Scene {
    WindowGroup {
      #if DEBUG
      if ProcessInfo.processInfo.arguments.contains("--preview-large-text") {
        WatchViewer().environment(\.dynamicTypeSize, .accessibility3)
      } else { WatchViewer() }
      #else
      WatchViewer()
      #endif
    }
  }
}
