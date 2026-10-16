import CSketchFFI
import UIKit
import UniformTypeIdentifiers

extension DrawingController {
  func mayReplace(_ action: @escaping () -> Void) {
    canvas.cancelAll()
    guard editor.state.dirty != 0 else {
      action()
      return
    }
    let a = UIAlertController(
      title: "Save changes?",
      message: "Unsaved changes will be lost if you continue without saving.",
      preferredStyle: .alert)
    a.addAction(
      UIAlertAction(title: "Save…", style: .default) { [weak self] _ in
        self?.afterSave = action
        self?.saveDocument()
      })
    a.addAction(UIAlertAction(title: "Discard Changes", style: .destructive) { _ in action() })
    a.addAction(UIAlertAction(title: "Cancel", style: .cancel))
    present(a, animated: true)
  }
  @objc func newDocument() {
    mayReplace { [weak self] in
      guard let self else { return }
      if self.editor.handle == nil { self.editor.reset() }
      if self.editor.check(sketch_document_new(self.editor.handle)) {
        self.name = "Untitled"
        #if os(iOS)
          self.watchSender.documentID = UUID().uuidString
        #endif
        self.canvas.zoom = 1
        self.canvas.offset = CGPoint(x: 32, y: 32)
      }
      self.refresh()
    }
  }
  @objc func openDocument() {
    mayReplace { [weak self] in
      guard let self else { return }
      let picker = UIDocumentPickerViewController(forOpeningContentTypes: [.data], asCopy: true)
      picker.delegate = self
      picker.allowsMultipleSelection = false
      self.present(picker, animated: true)
    }
  }
  @objc func saveDocument() {
    canvas.cancelAll()
    guard let (data, revision) = editor.snapshot() else {
      afterSave = nil
      return
    }
    do {
      let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
      try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
      let url = dir.appendingPathComponent(name == "Untitled" ? "Untitled.nsketch" : name)
      try data.write(to: url, options: .atomic)
      exportURL = url
      exportRevision = revision
      let picker = UIDocumentPickerViewController(forExporting: [url], asCopy: true)
      picker.delegate = self
      present(picker, animated: true)
    } catch {
      afterSave = nil
      self.error(error.localizedDescription)
    }
  }
  func cleanupExport() {
    if let exportURL {
      try? FileManager.default.removeItem(at: exportURL.deletingLastPathComponent())
    }
    exportURL = nil
    exportRevision = nil
  }
  func documentPickerWasCancelled(_ controller: UIDocumentPickerViewController) {
    cleanupExport()
    afterSave = nil
    refresh()
  }
  func documentPicker(_ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL])
  {
    controller.dismiss(animated: true) { [weak self] in self?.finishPicking(urls) }
  }
  // Stream imports so untrusted document-provider files cannot allocate beyond Rust's
  // 64 MiB document ceiling. Metadata is advisory and is deliberately not trusted.
  static func readDocument(_ url: URL, limit: Int = 64 * 1024 * 1024) throws -> Data {
    guard let stream = InputStream(url: url) else { throw CocoaError(.fileReadUnknown) }
    stream.open()
    defer { stream.close() }
    var result = Data()
    var buffer = [UInt8](repeating: 0, count: 64 * 1024)
    while true {
      let remaining = limit - result.count
      let count = stream.read(&buffer, maxLength: min(buffer.count, remaining + 1))
      if count < 0 { throw stream.streamError ?? CocoaError(.fileReadUnknown) }
      if count == 0 { return result }
      guard count <= remaining else { throw CocoaError(.fileReadTooLarge) }
      result.append(contentsOf: buffer.prefix(count))
    }
  }
  func finishPicking(_ urls: [URL]) {
    guard let url = urls.first else {
      cleanupExport()
      afterSave = nil
      return
    }
    if let revision = exportRevision {
      let saved = editor.check(sketch_document_mark_saved(editor.handle, revision))
      if saved { name = url.lastPathComponent }
      cleanupExport()
      let action = afterSave
      afterSave = nil
      refresh()
      if saved { action?() }
      return
    }
    let scoped = url.startAccessingSecurityScopedResource()
    defer { if scoped { url.stopAccessingSecurityScopedResource() } }
    do {
      let data = try Self.readDocument(url)
      if editor.handle == nil {
        editor.reset()
        name = "Untitled"
      }
      if editor.load(data) {
        name = url.lastPathComponent
        #if os(iOS)
          watchSender.documentID = UUID().uuidString
        #endif
        canvas.zoom = 1
        canvas.offset = CGPoint(x: 32, y: 32)
      }
      refresh()
    } catch { self.error(error.localizedDescription) }
  }
}
