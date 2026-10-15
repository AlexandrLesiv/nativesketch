import AppKit
import CSketchFFI

@MainActor final class EditorMenuController: NSObject, NSMenuItemValidation {
  unowned let owner: EditorWindowController
  init(owner: EditorWindowController) { self.owner = owner }
  func makeMenu() {
    let bar = NSMenu()
    NSApp.mainMenu = bar
    func menu(_ title: String) -> NSMenu {
      let item = NSMenuItem()
      item.title = title
      item.submenu = NSMenu(title: title)
      bar.addItem(item)
      return item.submenu!
    }
    func add(
      _ m: NSMenu, _ title: String, _ action: Selector, _ key: String = "",
      _ flags: NSEvent.ModifierFlags = .command
    ) {
      let item = m.addItem(withTitle: title, action: action, keyEquivalent: key)
      item.keyEquivalentModifierMask = flags
      item.target = target(for: action)
    }
    add(menu("Native Sketch"), "Quit Native Sketch", #selector(EditorMenuController.quit(_:)), "q")
    let file = menu("File")
    add(file, "New", #selector(DocumentCoordinator.newDocument(_:)), "n")
    add(file, "Open…", #selector(DocumentCoordinator.openDocument(_:)), "o")
    add(file, "Save", #selector(DocumentCoordinator.saveDocument(_:)), "s")
    add(file, "Save As…", #selector(DocumentCoordinator.saveAs(_:)), "s", [.command, .shift])
    add(file, "Close", #selector(EditorMenuController.closeWindow(_:)), "w")
    let edit = menu("Edit")
    add(edit, "Undo", #selector(EditorActions.undo(_:)), "z")
    add(edit, "Redo", #selector(EditorActions.redo(_:)), "z", [.command, .shift])
    edit.addItem(.separator())
    for (name, action, key) in [
      ("Cut", #selector(NSText.cut(_:)), "x"), ("Copy", #selector(NSText.copy(_:)), "c"),
      ("Paste", #selector(NSText.paste(_:)), "v"),
      ("Select All", #selector(NSText.selectAll(_:)), "a"),
    ] { edit.addItem(withTitle: name, action: action, keyEquivalent: key) }
    add(edit, "Select Next Stroke", #selector(EditorActions.nextStroke(_:)), "]")
    add(edit, "Select Previous Stroke", #selector(EditorActions.previousStroke(_:)), "[")
    add(edit, "Delete Stroke", #selector(EditorActions.deleteStroke(_:)))
    let tools = menu("Tools")
    add(tools, "Pen", #selector(EditorActions.penTool(_:)))
    add(tools, "Select", #selector(EditorActions.selectTool(_:)))
    let view = menu("View")
    add(view, "Zoom In", #selector(EditorActions.zoomIn(_:)), "+")
    add(view, "Zoom Out", #selector(EditorActions.zoomOut(_:)), "-")
    add(view, "Actual Size", #selector(EditorActions.actualSize(_:)), "0")
  }
  @objc func quit(_ sender: Any?) { NSApp.terminate(sender) }
  @objc func closeWindow(_ sender: Any?) { owner.window.performClose(sender) }
  func validateMenuItem(_ item: NSMenuItem) -> Bool {
    if owner.editor.unavailable {
      return [
        #selector(DocumentCoordinator.newDocument(_:)),
        #selector(DocumentCoordinator.openDocument(_:)),
        #selector(EditorMenuController.closeWindow(_:)),
        #selector(EditorMenuController.quit(_:)),
      ].contains(item.action)
    }
    let s = owner.editor.state
    if item.action == #selector(EditorActions.undo(_:)) {
      return (owner.window?.firstResponder as? NSTextView)?.undoManager?.canUndo
        ?? (s.can_undo != 0)
    }
    if item.action == #selector(EditorActions.redo(_:)) {
      return (owner.window?.firstResponder as? NSTextView)?.undoManager?.canRedo
        ?? (s.can_redo != 0)
    }
    if item.action == #selector(EditorActions.deleteStroke(_:)) {
      return s.selected_id != 0 && !(owner.window?.firstResponder is NSTextView)
    }
    if item.action == #selector(EditorActions.nextStroke(_:))
      || item.action == #selector(EditorActions.previousStroke(_:))
    {
      return s.stroke_count > 0
    }
    return true
  }
  func target(for action: Selector) -> AnyObject {
    if [
      #selector(DocumentCoordinator.newDocument(_:)),
      #selector(DocumentCoordinator.openDocument(_:)),
      #selector(DocumentCoordinator.saveDocument(_:)), #selector(DocumentCoordinator.saveAs(_:)),
    ].contains(action) {
      return owner.documents
    }
    if [#selector(EditorMenuController.quit(_:)), #selector(EditorMenuController.closeWindow(_:))]
      .contains(action)
    {
      return self
    }
    return owner.actions
  }
}
