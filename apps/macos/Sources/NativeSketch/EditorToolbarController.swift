import AppKit
import CSketchSession

@MainActor final class EditorToolbarController: NSObject, NSToolbarDelegate {
  unowned let owner: EditorWindowController
  init(owner: EditorWindowController) { self.owner = owner }
  let toolToolbarID = NSToolbarItem.Identifier("tool")
  let editingToolbarID = NSToolbarItem.Identifier("editing")
  func toolbarDefaultItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] {
    [toolToolbarID, editingToolbarID]
  }
  func toolbarAllowedItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] {
    [toolToolbarID, editingToolbarID, .flexibleSpace]
  }
  func toolbar(
    _ toolbar: NSToolbar, itemForItemIdentifier identifier: NSToolbarItem.Identifier,
    willBeInsertedIntoToolbar: Bool
  ) -> NSToolbarItem? {
    let item = NSToolbarItem(itemIdentifier: identifier)
    item.visibilityPriority = .high
    if identifier == toolToolbarID {
      item.label = "Drawing tool"
      item.view = owner.inspectorController.tool
    } else if identifier == editingToolbarID {
      item.label = "Style and Selection"
      item.toolTip = "Edit stroke color, width, and selection scale"
      item.image = NSImage(
        systemSymbolName: "slider.horizontal.3", accessibilityDescription: "Style and Selection")
      item.target = owner.inspectorController
      item.action = #selector(BrushInspectorController.showInspector(_:))
      item.isBordered = true
    } else {
      return nil
    }
    return item
  }
}
