import Darwin
import SwiftUI
import WatchConnectivity

@MainActor final class WatchModel: NSObject, ObservableObject, WCSessionDelegate {
  @Published var navigation = ViewerNavigation()
  @Published var message = "Send a copy from iPhone"
  @Published var version = 0
  let store = WatchCopyStore(cache: FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0].appendingPathComponent("LastCopy.json"))
  override init() {
    super.init()
    do { try store.restore(); refresh() } catch { message = "Cached copy could not be opened. Send a copy from iPhone." }
    if WCSession.isSupported() { WCSession.default.delegate = self; WCSession.default.activate() }
    #if DEBUG
    if ProcessInfo.processInfo.arguments.contains("--integration-test") {
      do { try ViewerReplay.run(); print("APPLE VIEWER REPLAY PASS"); exit(0) }
      catch { print("APPLE VIEWER REPLAY FAIL: \(error)"); exit(1) }
    }
    if ProcessInfo.processInfo.arguments.contains("--preview-sample") {
      do {
        guard let url = Bundle.main.url(forResource: "Sample", withExtension: "nsketch") else { throw ViewerError.invalid }
        let title = String(repeating: "A long simulator drawing title · ", count: 8)
        let metadata = try WatchCopyMetadata(["protocolVersion": 1, "sentAt": max(Date().timeIntervalSince1970, (store.metadata?.sentAt ?? 0) + 1), "title": title, "contentType": "dev.nativesketch.document"])
        try store.receive(boundedRead(url, limit: WatchCopyStore.limit), metadata: metadata)
        refresh(); message = "Simulator sample · No paired transfer"
      } catch { message = "Simulator fixture failed: \(error.localizedDescription)" }
    }
    #endif
  }
  func refresh() {
    if store.metadata != nil { message = "" }
    version += 1
  }
  nonisolated func session(_ session: WCSession, activationDidCompleteWith activationState: WCSessionActivationState, error: Error?) {
    if let error { let text = error.localizedDescription; Task { @MainActor in self.message = text } }
  }
  nonisolated func session(_ session: WCSession, didReceive file: WCSessionFile) {
    // WCSession deletes its temporary URL after this callback: take ownership first.
    let retained = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString).appendingPathExtension("nsketch")
    do {
      let metadata = try WatchCopyMetadata(file.metadata ?? [:])
      try FileManager.default.moveItem(at: file.fileURL, to: retained)
      Task { @MainActor in
        defer { try? FileManager.default.removeItem(at: retained) }
        do { try self.store.receive(boundedRead(retained, limit: WatchCopyStore.limit), metadata: metadata); self.navigation.fit(); self.refresh() }
        catch { self.message = "\(error.localizedDescription) Previous copy retained." }
      }
    } catch { let text = error.localizedDescription; Task { @MainActor in self.message = "\(text) Previous copy retained." } }
  }
}
struct WatchViewer: View {
  @StateObject private var model = WatchModel()
  @State private var crown = 1.0
  @State private var startPan: CGSize?
  @State private var showingDetails = false
  @Environment(\.dynamicTypeSize) private var dynamicTypeSize
  private var detailsSummary: String {
    let received = model.store.receivedAt.map { "Received " + Date(timeIntervalSince1970: $0).formatted(date: .abbreviated, time: .shortened) } ?? ""
    return [model.store.metadata?.title ?? "", received, model.message].filter { !$0.isEmpty }.joined(separator: ". ")
  }
  var body: some View {
    Group {
      if dynamicTypeSize.isAccessibilitySize {
        // Large text can scroll to every native control without shrinking the canvas.
        ScrollView { primary.frame(minHeight: 280) }
      } else { primary }
    }
    .sheet(isPresented: $showingDetails) { details }
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
        if model.store.metadata == nil {
          Text("Send a copy from iPhone").font(.caption2).foregroundStyle(.black)
            .multilineTextAlignment(.center).padding(6).allowsHitTesting(false)
        }
      }
      .accessibilityLabel("Drawing preview, view only")
      .focusable()
      .digitalCrownRotation($crown, from: 0.25, through: 16, by: 0.1, sensitivity: .low, isContinuous: false)
      .onChange(of: model.version) { _, _ in crown = 1; startPan = nil }
      .onChange(of: crown) { _, value in model.navigation.zoom = value }
      .gesture(DragGesture().onChanged { value in
        if startPan == nil { startPan = model.navigation.pan }
        model.navigation.pan = CGSize(width: (startPan?.width ?? 0) + value.translation.width, height: (startPan?.height ?? 0) + value.translation.height)
      }.onEnded { _ in startPan = nil })
      if let metadata = model.store.metadata, !dynamicTypeSize.isAccessibilitySize {
        ViewThatFits(in: .vertical) {
          Text(metadata.title).font(.caption2).lineLimit(1).truncationMode(.tail)
            .accessibilityLabel("Document: " + metadata.title)
          EmptyView()
        }
      }
      HStack {
        Button("Fit") { crown = 1; model.navigation.fit() }
          .disabled(model.store.document.paths.isEmpty)
        Button("Details") { showingDetails = true }
          .accessibilityValue(detailsSummary)
      }
    }.padding(4)
  }
  private var details: some View {
      ScrollView {
        VStack(alignment: .leading, spacing: 12) {
          Text("View Only").font(.headline)
          if let metadata = model.store.metadata {
            Text(metadata.title).fixedSize(horizontal: false, vertical: true)
            Text("Received " + Date(timeIntervalSince1970: model.store.receivedAt ?? metadata.sentAt).formatted(date: .complete, time: .shortened))
          }
          if !model.message.isEmpty { Text(model.message).fixedSize(horizontal: false, vertical: true) }
          Text("Turn the Digital Crown to zoom. Drag to pan. Fit shows the entire drawing.")
          Button("Done") { showingDetails = false }
        }.padding()
      }
  }
}
@main struct NativeSketchWatch: App { var body: some Scene { WindowGroup { WatchViewer() } } }
