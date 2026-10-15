import AppKit
import CSketchFFI

@MainActor final class DocumentCoordinator: NSObject, NSMenuItemValidation {
  unowned let owner: EditorWindowController
  init(owner: EditorWindowController) { self.owner = owner }
  var url: URL?
  var approvedClose = false
  func save(forcePanel: Bool = false) -> Bool {
    owner.canvas.cancel()
    owner.window.makeFirstResponder(owner.canvas)
    var destination = url
    if destination == nil || forcePanel {
      let panel = NSSavePanel()
      panel.nameFieldStringValue = url?.lastPathComponent ?? "Untitled.nsketch"
      guard panel.runModal() == .OK else { return false }
      destination = panel.url
    }
    guard let destination else { return false }
    let success = owner.failures.checkFile(
      owner.editor.file(destination, save: true), url: destination, verb: "save")
    if success { url = destination }
    owner.refresh()
    return success
  }
  func mayReplace() -> Bool {
    owner.canvas.cancel()
    guard owner.editor.state.dirty != 0 else { return true }
    let alert = NSAlert()
    alert.messageText = "Save changes to \(url?.lastPathComponent ?? "Untitled")?"
    alert.informativeText = "Your changes will be lost if you don’t save them."
    alert.addButton(withTitle: "Save")
    alert.addButton(withTitle: "Cancel")
    alert.addButton(withTitle: "Don’t Save")
    switch alert.runModal() {
    case .alertFirstButtonReturn: return save()
    case .alertThirdButtonReturn: return true
    default: return false
    }
  }
  @objc func saveDocument(_ sender: Any?) { _ = save() }
  @objc func saveAs(_ sender: Any?) { _ = save(forcePanel: true) }
  func recoverEditorIfNeeded() {
    guard owner.editor.unavailable else { return }
    // A new empty editor must never inherit the previous document destination.
    url = nil
    owner.editor.reset()
    owner.failures.reportedEngineFailure = false
  }
  @objc func newDocument(_ sender: Any?) {
    guard mayReplace() else { return }
    recoverEditorIfNeeded()
    if owner.failures.check(sketch_document_new(owner.editor.handle)) {
      url = nil
      owner.canvas.zoom = 1
      owner.canvas.offset = CGPoint(x: 80, y: 80)
    }
    owner.refresh()
  }
  @objc func openDocument(_ sender: Any?) {
    guard mayReplace() else { return }
    let panel = NSOpenPanel()
    panel.allowsMultipleSelection = false
    guard panel.runModal() == .OK, let destination = panel.url else { return }
    recoverEditorIfNeeded()
    if owner.failures.checkFile(
      owner.editor.file(destination, save: false), url: destination, verb: "open")
    {
      url = destination
      owner.canvas.zoom = 1
      owner.canvas.offset = CGPoint(x: 80, y: 80)
    }
    owner.refresh()
  }
  func validateMenuItem(_ item: NSMenuItem) -> Bool { owner.menus.validateMenuItem(item) }
}
