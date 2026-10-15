import AppKit
import CSketchSession

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
    add(file, "Recover Drawings…", #selector(DocumentCoordinator.showRecovery(_:)))
    add(file, "Cancel File Operation", #selector(DocumentCoordinator.cancelNavigation(_:)))
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
    if item.action == #selector(EditorMenuController.closeWindow(_:)), owner.editor.unavailable {
      return true  // Terminal native cleanup remains available after a poisoned engine.
    }
    if item.action == #selector(EditorActions.undo(_:)), let field = owner.actions.fieldEditor {
      return field.undoManager?.canUndo == true
    }
    if item.action == #selector(EditorActions.redo(_:)), let field = owner.actions.fieldEditor {
      return field.undoManager?.canRedo == true
    }
    if item.action == #selector(EditorActions.deleteStroke(_:)), owner.actions.fieldEditor != nil {
      return false
    }
    if item.action == #selector(DocumentCoordinator.cancelNavigation(_:)) {
      return owner.editor.projection.continuation.serial != 0
    }
    if item.action == #selector(EditorActions.nextStroke(_:))
      || item.action == #selector(EditorActions.previousStroke(_:))
    {
      return owner.editor.projection.stroke_count > 0
        && owner.editor.available(SK_V1_CHOOSE_SELECT.rawValue)
    }
    let mapping: [Selector: UInt32] = [
      #selector(EditorActions.undo(_:)): SK_V1_UNDO.rawValue,
      #selector(EditorActions.redo(_:)): SK_V1_REDO.rawValue,
      #selector(EditorActions.deleteStroke(_:)): SK_V1_DELETE_SELECTION.rawValue,
      #selector(EditorActions.penTool(_:)): SK_V1_CHOOSE_PEN.rawValue,
      #selector(EditorActions.selectTool(_:)): SK_V1_CHOOSE_SELECT.rawValue,
      #selector(EditorActions.nextStroke(_:)): SK_V1_SELECT_CYCLE.rawValue,
      #selector(EditorActions.previousStroke(_:)): SK_V1_SELECT_CYCLE.rawValue,
      #selector(DocumentCoordinator.saveDocument(_:)): SK_V1_SAVE.rawValue,
      #selector(DocumentCoordinator.saveAs(_:)): SK_V1_SAVE_AS.rawValue,
      #selector(DocumentCoordinator.newDocument(_:)): SK_V1_NEW.rawValue,
      #selector(DocumentCoordinator.openDocument(_:)): SK_V1_OPEN.rawValue,
      #selector(EditorMenuController.closeWindow(_:)): SK_V1_CLOSE.rawValue,
    ]
    if let action = item.action, let tag = mapping[action] { return owner.editor.available(tag) }
    return true
  }
  func target(for action: Selector) -> AnyObject {
    if [
      #selector(DocumentCoordinator.showRecovery(_:)),
      #selector(DocumentCoordinator.cancelNavigation(_:)),
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
