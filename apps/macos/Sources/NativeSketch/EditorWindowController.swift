import AppKit
import CSketchFFI

@MainActor final class EditorWindowController: NSObject, NSWindowDelegate {
  let editor = LegacyEditorBridge()
  var window: NSWindow!
  var canvas: CanvasView!
  let status = NSTextField(labelWithString: "")
  lazy var actions = EditorActions(owner: self)
  lazy var inspectorController = BrushInspectorController(owner: self)
  lazy var documents = DocumentCoordinator(owner: self)
  lazy var failures = EngineFailurePresenter(owner: self)
  lazy var menus = EditorMenuController(owner: self)
  lazy var toolbarController = EditorToolbarController(owner: self)
  func show() {
    editor.onFailure = { [weak self] code in
      Task { @MainActor in self?.failures.engineFailure(code) }
    }
    menus.makeMenu()
    window = NSWindow(
      contentRect: NSRect(x: 0, y: 0, width: 1000, height: 700),
      styleMask: [.titled, .closable, .miniaturizable, .resizable], backing: .buffered, defer: false
    )
    if let index = CommandLine.arguments.firstIndex(of: "--appearance"),
      CommandLine.arguments.count > index + 1
    {
      window.appearance = NSAppearance(
        named: CommandLine.arguments[index + 1] == "dark" ? .darkAqua : .aqua)
    }
    if CommandLine.arguments.contains("--small-window") {
      window.setContentSize(NSSize(width: 640, height: 420))
    }
    window.title = "Untitled — Native Sketch"
    window.minSize = NSSize(width: 640, height: 420)
    window.isReleasedWhenClosed = false
    window.delegate = self
    canvas = CanvasView(owner: self)
    inspectorController.configure()
    let toolbar = NSToolbar(identifier: "NativeSketchEditing")
    toolbar.delegate = toolbarController
    toolbar.displayMode = .iconOnly
    toolbar.allowsUserCustomization = false
    window.toolbar = toolbar
    window.toolbarStyle = .unified
    status.font = .systemFont(ofSize: 12)
    status.textColor = .secondaryLabelColor
    status.setAccessibilityLabel("Document status")
    let root = NSView()
    window.contentView = root
    for v in [canvas!, status] {
      v.translatesAutoresizingMaskIntoConstraints = false
      root.addSubview(v)
    }
    NSLayoutConstraint.activate([
      canvas.topAnchor.constraint(equalTo: root.topAnchor),
      canvas.leadingAnchor.constraint(equalTo: root.leadingAnchor),
      canvas.trailingAnchor.constraint(equalTo: root.trailingAnchor),
      canvas.bottomAnchor.constraint(equalTo: status.topAnchor, constant: -6),
      status.leadingAnchor.constraint(equalTo: root.leadingAnchor, constant: 12),
      status.trailingAnchor.constraint(equalTo: root.trailingAnchor, constant: -12),
      status.bottomAnchor.constraint(equalTo: root.bottomAnchor, constant: -8),
    ])
    let focusOrder: [NSView] = [
      self.inspectorController.tool, self.inspectorController.color, self.inspectorController.width,
      self.inspectorController.stepper, self.inspectorController.scale,
      self.inspectorController.apply, canvas,
    ]
    for index in focusOrder.indices {
      focusOrder[index].nextKeyView = focusOrder[(index + 1) % focusOrder.count]
    }
    refresh()
    window.center()
    window.makeKeyAndOrderFront(nil)
    window.makeFirstResponder(canvas)
    NSApp.activate(ignoringOtherApps: true)
    if CommandLine.arguments.contains("--native-smoke-test") {
      DispatchQueue.main.async { NativeSmokeJourney(owner: self).run() }
    }
  }
  func refresh() {
    guard canvas != nil else { return }
    let s = editor.state
    window.isDocumentEdited = s.dirty != 0
    window.title = "\(documents.url?.lastPathComponent ?? "Untitled") — Native Sketch"
    for control in [
      self.inspectorController.tool, self.inspectorController.color, self.inspectorController.width,
      self.inspectorController.stepper,
    ] as [NSControl] {
      control.isEnabled = !editor.unavailable
    }
    if editor.unavailable {
      self.inspectorController.scale.isEnabled = false
      self.inspectorController.apply.isEnabled = false
      status.stringValue = "Engine unavailable — use File > New or Open."
      canvas.needsDisplay = true
      return
    }
    let selected = s.selected_id != 0
    self.inspectorController.scale.isEnabled = selected
    self.inspectorController.apply.isEnabled = selected
    status.stringValue =
      "\(s.stroke_count) stroke\(s.stroke_count == 1 ? "" : "s") · \(selected ? "Stroke selected" : inspectorController.tool.selectedSegment == 0 ? "Pen — drag to draw" : "Select — click a stroke") · \(Int(canvas.zoom*100))% · Space-drag to pan"
    canvas.setAccessibilityValue(
      "\(s.stroke_count) strokes, \(selected ? "one selected" : "none selected"), zoom \(Int(canvas.zoom*100)) percent"
    )
    canvas.needsDisplay = true
  }
  func windowShouldClose(_ sender: NSWindow) -> Bool {
    self.documents.approvedClose = self.documents.mayReplace()
    return self.documents.approvedClose
  }
  func windowDidResignKey(_ notification: Notification) { canvas?.cancel() }
}
