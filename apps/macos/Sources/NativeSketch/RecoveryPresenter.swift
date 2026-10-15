import AppKit
import CSketchSession

@MainActor final class RecoveryPresenter: NSObject, NSTableViewDataSource, NSTableViewDelegate {
  unowned let owner: EditorWindowController
  private let panel = NSPanel(
    contentRect: NSRect(x: 0, y: 0, width: 580, height: 360),
    styleMask: [.titled], backing: .buffered, defer: false)
  private let table = NSTableView()
  private let status = NSTextField(labelWithString: "Loading recovery copies…")
  private let open = NSButton(title: "Open Recovered Copy", target: nil, action: nil)
  private let discard = NSButton(title: "Discard…", target: nil, action: nil)
  private var requestedItem: UInt64?
  private var inventory: RecoveryInventory { owner.runtime.recovery }
  init(owner: EditorWindowController) {
    self.owner = owner
    super.init()
  }

  func show() {
    guard owner.window.attachedSheet == nil else { return }
    let focusGeneration = owner.focusRestoration.capture()
    let session = owner.editor.projection.session
    guard owner.inspectorController.fieldEditing.finishEditing() else {
      owner.focusRestoration.discard()
      return
    }
    configure()
    status.stringValue = "Loading recovery copies…"
    inventory.onChange = { [weak self] in self?.reload() }
    inventory.onFailure = { [weak self] failure in
      self?.status.stringValue =
        "Recovery operation failed (\(failure.code)). Existing copies are retained."
    }
    owner.window.beginSheet(panel) { [weak self] _ in
      guard let self else { return }
      inventory.onChange = nil
      inventory.onFailure = nil
      if let requestedItem {
        _ = owner.documents.request(.recover(requestedItem), preservingFocus: focusGeneration)
      } else {
        owner.focusRestoration.cancelled(session: session)
      }
      requestedItem = nil
      inventory.dismiss()
      // NativeEffectPresenter observes the parent's didEndSheet lifecycle before
      // presenting any retained failure; this callback may precede detachment.
    }
    inventory.refresh()
  }
  private func configure() {
    guard panel.contentView?.subviews.isEmpty != false else { return }
    panel.title = "Recover Drawings"
    status.setAccessibilityIdentifier("sketch.recovery.status")
    let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("drawing"))
    column.title = "Recovery copy"
    column.width = 520
    table.addTableColumn(column)
    table.delegate = self
    table.dataSource = self
    table.setAccessibilityIdentifier("sketch.recovery.list")
    let scroll = NSScrollView()
    scroll.documentView = table
    scroll.hasVerticalScroller = true
    scroll.heightAnchor.constraint(equalToConstant: 240).isActive = true
    open.target = self
    open.action = #selector(openSelected)
    open.setAccessibilityIdentifier("sketch.recovery.open")
    discard.target = self
    discard.action = #selector(discardSelected)
    discard.setAccessibilityIdentifier("sketch.recovery.discard")
    let later = NSButton(title: "Later", target: self, action: #selector(dismiss))
    later.keyEquivalent = "\u{1b}"
    later.setAccessibilityIdentifier("sketch.recovery.later")
    let retry = NSButton(title: "Retry Recovery", target: self, action: #selector(retryRecovery))
    retry.setAccessibilityIdentifier("sketch.recovery.retry")
    let buttons = NSStackView(views: [open, discard, retry, later])
    let stack = NSStackView(views: [status, scroll, buttons])
    stack.orientation = .vertical
    stack.alignment = .leading
    stack.spacing = 12
    stack.edgeInsets = NSEdgeInsets(top: 16, left: 16, bottom: 16, right: 16)
    stack.translatesAutoresizingMaskIntoConstraints = false
    panel.contentView?.addSubview(stack)
    if let content = panel.contentView {
      NSLayoutConstraint.activate([
        stack.leadingAnchor.constraint(equalTo: content.leadingAnchor),
        stack.trailingAnchor.constraint(equalTo: content.trailingAnchor),
        stack.topAnchor.constraint(equalTo: content.topAnchor),
        stack.bottomAnchor.constraint(equalTo: content.bottomAnchor),
        scroll.widthAnchor.constraint(equalTo: stack.widthAnchor, constant: -32),
      ])
    }
    reload()
  }
  private func reload() {
    table.reloadData()
    status.stringValue =
      inventory.info.capacity_blocked != 0
      ? "Recovery capacity is full. Existing copies are retained."
      : inventory.rows.isEmpty
        ? "No recovery copies are available."
        : "\(inventory.rows.count) recovery copies. Opening a copy does not delete it."
    updateSelection()
  }
  func numberOfRows(in tableView: NSTableView) -> Int { inventory.rows.count }
  func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView?
  {
    guard inventory.rows.indices.contains(row) else { return nil }
    let item = inventory.rows[row]
    return NSTextField(
      labelWithString:
        "\(item.name.isEmpty ? "Untitled" : item.name) — revision \(item.value.revision)\(item.value.rejected_generations > 0 ? " (earlier valid copy)" : "")"
    )
  }
  func tableViewSelectionDidChange(_ notification: Notification) { updateSelection() }
  private func updateSelection() {
    let selected = inventory.rows.indices.contains(table.selectedRow)
    open.isEnabled = selected
    discard.isEnabled = selected
  }
  @objc private func openSelected() {
    guard inventory.rows.indices.contains(table.selectedRow) else { return }
    requestedItem = inventory.rows[table.selectedRow].value.item_id
    owner.window.endSheet(panel)
  }
  @objc private func dismiss() { owner.window.endSheet(panel) }
  @objc private func retryRecovery() { inventory.retry(window: owner.editor.projection.window) }
  @objc private func discardSelected() {
    guard inventory.rows.indices.contains(table.selectedRow) else { return }
    let id = inventory.rows[table.selectedRow].value.item_id
    let alert = NSAlert()
    alert.messageText = "Discard this recovery copy?"
    alert.informativeText = "This removes only the selected recovery generation."
    alert.addButton(withTitle: "Cancel").setAccessibilityIdentifier(
      "sketch.recovery.confirm-cancel")
    alert.addButton(withTitle: "Discard").setAccessibilityIdentifier(
      "sketch.recovery.confirm-discard")
    alert.beginSheetModal(for: panel) { [weak self] response in
      if response == .alertSecondButtonReturn { self?.inventory.discard(id) }
    }
  }
}
