import AppKit
import CSketchSession

@MainActor final class EditorWindowController: NSObject, NSWindowDelegate {
  let editor: RustApplicationSession
  let runtime: NativeRuntime
  lazy var scene = SessionScene(session: editor)
  lazy var presenter = EditorPresenter(owner: self)
  lazy var lifecycle = WindowLifecycleCoordinator(owner: self)
  lazy var recoveryPresenter = RecoveryPresenter(owner: self)
  init(runtime: NativeRuntime) throws {
    self.runtime = runtime
    self.editor = try RustApplicationSession(runtime: runtime)
    super.init()
  }
  var window: NSWindow!
  var canvas: CanvasView!
  var uiObservation: NativeUIObservation?
  let status = NSTextField(labelWithString: "")
  lazy var actions = EditorActions(owner: self)
  lazy var inspectorController = BrushInspectorController(owner: self)
  lazy var focusRestoration = NativeFocusRestoration(owner: self)
  lazy var documents = DocumentCoordinator(owner: self)
  lazy var failures = EngineFailurePresenter(owner: self)
  lazy var menus = EditorMenuController(owner: self)
  lazy var toolbarController = EditorToolbarController(owner: self)
  func show() {
    editor.onFailure = { [weak self] code in
      Task { @MainActor in self?.failures.engineFailure(code) }
    }
    editor.onOutcome = { [weak self] outcome in self?.presenter.receive(outcome) }
    runtime.onFailure = { [weak self] failure in
      NativeDiagnostics.failure(
        "runtime-background", status: failure.code, session: failure.session)
      self?.status.stringValue =
        "Background storage is unavailable. Your open document is retained."
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
    uiObservation = NativeUIObservation(canvas: canvas)
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
    status.setAccessibilityIdentifier("sketch.status")
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
    editor.registerRecovery(name: documents.displayName)
    window.makeKeyAndOrderFront(nil)
    window.makeFirstResponder(canvas)
    NSApp.activate(ignoringOtherApps: true)
    if CommandLine.arguments.contains("--native-smoke-test") {
      DispatchQueue.main.async { NativeSmokeJourney(owner: self).run() }
    }
  }
  func refresh() { presenter.refresh() }
  func windowShouldClose(_ sender: NSWindow) -> Bool { lifecycle.requestWindowClose() }
  func windowDidResignKey(_ notification: Notification) { canvas?.cancel() }
}
