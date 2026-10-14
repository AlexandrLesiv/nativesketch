import AppKit
import CSketchFFI

// All adapter/editor access is confined to AppKit's main actor. Core owns editing,
// history, hit testing, pressure geometry, and persistence. This painter is provisional.
@MainActor final class Editor {
  var handle: OpaquePointer
  init() {
    var h: OpaquePointer?
    precondition(sketch_editor_create(&h) == 0)
    handle = h!
  }
  deinit { sketch_editor_free(handle) }
  var state: SketchState {
    var s = SketchState()
    _ = sketch_editor_state(handle, &s)
    return s
  }
  func file(_ url: URL, save: Bool) -> UInt32 {
    let bytes = Array(url.path.utf8)
    return bytes.withUnsafeBufferPointer {
      save
        ? sketch_document_save(handle, $0.baseAddress, $0.count)
        : sketch_document_load(handle, $0.baseAddress, $0.count)
    }
  }
  func commands() -> [SketchPathCommand] {
    var count = 0
    guard sketch_render_copy(handle, nil, 0, &count) == 0 else { return [] }
    var commands = Array(repeating: SketchPathCommand(), count: count)
    let result = commands.withUnsafeMutableBufferPointer {
      sketch_render_copy(handle, $0.baseAddress, $0.count, &count)
    }
    return result == 0 ? commands : []
  }
}

@MainActor final class Canvas: NSView {
  unowned let owner: AppDelegate
  var zoom = 1.0
  var offset = CGPoint(x: 80, y: 80)
  var space = false
  var gesture = 0  // 0 idle, 1 ink, 2 resize, 3 pan
  var sampleID: UInt64 = 0
  var start = CGPoint.zero
  var anchor = CGPoint.zero
  var initialVector = CGPoint.zero
  init(owner: AppDelegate) {
    self.owner = owner
    super.init(frame: .zero)
    setAccessibilityElement(true)
    setAccessibilityRole(.group)
    setAccessibilityLabel("Drawing canvas")
    setAccessibilityHelp(
      "P for Pen, V for Select. Space drag to pan. Select strokes using the Edit menu, then use Scale percent to resize."
    )
  }
  required init?(coder: NSCoder) { fatalError() }
  override var isFlipped: Bool { true }
  override var acceptsFirstResponder: Bool { true }
  var editor: Editor { owner.editor }
  func document(_ p: CGPoint) -> CGPoint {
    CGPoint(x: (p.x - offset.x) / zoom, y: (p.y - offset.y) / zoom)
  }
  func screen(_ p: CGPoint) -> CGPoint {
    CGPoint(x: p.x * zoom + offset.x, y: p.y * zoom + offset.y)
  }
  func point(_ event: NSEvent) -> CGPoint { convert(event.locationInWindow, from: nil) }
  func cancel() {
    _ = sketch_cancel(editor.handle)
    gesture = 0
    space = false
    owner.refresh()
  }
  override func cancelOperation(_ sender: Any?) { escape() }
  func escape() {
    let idle = gesture == 0
    cancel()
    if idle {
      _ = sketch_select_clear(editor.handle)
      owner.refresh()
    }
  }
  override func viewDidChangeBackingProperties() {
    super.viewDidChangeBackingProperties()
    needsDisplay = true
  }
  override func setFrameSize(_ size: NSSize) {
    let old = frame.size
    if old != .zero && old != size {
      cancel()
      offset.x += (size.width - old.width) / 2
      offset.y += (size.height - old.height) / 2
    }
    super.setFrameSize(size)
  }
  override func draw(_ dirtyRect: NSRect) {
    NSColor.white.setFill()
    bounds.fill()
    guard let context = NSGraphicsContext.current?.cgContext else { return }
    context.saveGState()
    context.translateBy(x: offset.x, y: offset.y)
    context.scaleBy(x: zoom, y: zoom)
    var id: UInt64?
    var path = CGMutablePath()
    var color = NSColor.black.cgColor
    func flush() {
      context.addPath(path)
      context.setFillColor(color)
      context.fillPath()
    }
    for c in editor.commands() {
      if id != c.stroke_id {
        if id != nil { flush() }
        path = CGMutablePath()
        id = c.stroke_id
        color = NSColor(srgbRed: c.red, green: c.green, blue: c.blue, alpha: c.alpha).cgColor
      }
      switch c.verb {
      case 0: path.move(to: CGPoint(x: c.x1, y: c.y1))
      case 1: path.addLine(to: CGPoint(x: c.x1, y: c.y1))
      case 2:
        path.addCurve(
          to: CGPoint(x: c.x3, y: c.y3), control1: CGPoint(x: c.x1, y: c.y1),
          control2: CGPoint(x: c.x2, y: c.y2))
      default: path.closeSubpath()
      }
    }
    if id != nil { flush() }
    context.restoreGState()
    if editor.state.selected_id != 0 {
      let b = editor.state.selection
      let p = screen(CGPoint(x: b.x, y: b.y))
      let r = CGRect(x: p.x, y: p.y, width: b.width * zoom, height: b.height * zoom)
      NSColor.systemBlue.setStroke()
      let outline = NSBezierPath(rect: r)
      outline.lineWidth = 1.5
      outline.stroke()
      for p in corners(b) {
        let s = screen(p)
        NSColor.white.setFill()
        let h = NSBezierPath(rect: CGRect(x: s.x - 4, y: s.y - 4, width: 8, height: 8))
        h.fill()
        NSColor.systemBlue.setStroke()
        h.stroke()
      }
    }
    if window?.firstResponder === self {
      NSColor.keyboardFocusIndicatorColor.setStroke()
      let p = NSBezierPath(rect: bounds.insetBy(dx: 2, dy: 2))
      p.lineWidth = 2
      p.stroke()
    }
  }
  func corners(_ b: SketchBounds) -> [CGPoint] {
    [
      CGPoint(x: b.x, y: b.y), CGPoint(x: b.x + b.width, y: b.y),
      CGPoint(x: b.x + b.width, y: b.y + b.height), CGPoint(x: b.x, y: b.y + b.height),
    ]
  }
  func append(_ event: NSEvent) {
    sampleID += 1
    let p = document(point(event))
    let pen = event.subtype == .tabletPoint || event.type == .tabletPoint
    var sample = SketchSample(
      x: p.x, y: p.y, timestamp: event.timestamp, pressure: pen ? Double(event.pressure) : 0,
      sample_id: sampleID, has_pressure: pen ? 1 : 0, kind: 0)
    let result = sketch_stroke_append(editor.handle, 1, &sample, 1)
    if result != 0 {
      cancel()
      owner.status.stringValue = "Input cancelled because the engine rejected a sample (\(result))."
    }
    needsDisplay = true
  }
  override func mouseDown(with event: NSEvent) {
    window?.makeFirstResponder(self)
    start = point(event)
    if space {
      gesture = 3
      return
    }
    if owner.tool.selectedSegment == 0 {
      let color = owner.color.color.usingColorSpace(.sRGB) ?? .black
      let brush = SketchBrush(
        red: color.redComponent, green: color.greenComponent, blue: color.blueComponent,
        alpha: color.alphaComponent, width: owner.width.doubleValue)
      if owner.check(
        sketch_stroke_begin(editor.handle, 1, event.subtype == .tabletPoint ? 1 : 0, brush))
      {
        gesture = 1
        sampleID = 0
        append(event)
      }
    } else {
      let state = editor.state
      if state.selected_id != 0,
        let index = corners(state.selection).firstIndex(where: {
          let p = screen($0)
          return hypot(p.x - start.x, p.y - start.y) <= 9
        })
      {
        let points = corners(state.selection)
        anchor = points[(index + 2) % 4]
        let p = points[index]
        initialVector = CGPoint(x: p.x - anchor.x, y: p.y - anchor.y)
        if owner.check(sketch_resize_begin(editor.handle)) { gesture = 2 }
      } else {
        let p = document(start)
        _ = owner.check(sketch_select_at(editor.handle, p.x, p.y, 6 / zoom))
      }
    }
    owner.refresh()
  }
  override func mouseDragged(with event: NSEvent) {
    switch gesture {
    case 1: append(event)
    case 2:
      let p = document(point(event))
      let denominator = initialVector.x * initialVector.x + initialVector.y * initialVector.y
      let scale = max(
        0.01,
        ((p.x - anchor.x) * initialVector.x + (p.y - anchor.y) * initialVector.y)
          / max(denominator, 0.001))
      _ = owner.check(sketch_resize_update(editor.handle, scale, anchor.x, anchor.y))
      needsDisplay = true
    case 3:
      let p = point(event)
      offset.x += p.x - start.x
      offset.y += p.y - start.y
      start = p
      needsDisplay = true
    default: break
    }
  }
  override func mouseUp(with event: NSEvent) {
    if gesture == 1 {
      append(event)
      if gesture == 1 { _ = owner.check(sketch_stroke_commit(editor.handle, 1)) }
    }
    if gesture == 2 { _ = owner.check(sketch_resize_commit(editor.handle)) }
    gesture = 0
    owner.refresh()
  }
  override func scrollWheel(with event: NSEvent) {
    guard gesture == 0 else { return }
    offset.x -= event.scrollingDeltaX
    offset.y -= event.scrollingDeltaY
    needsDisplay = true
  }
  func setZoom(_ value: Double, at point: CGPoint? = nil) {
    guard gesture == 0 else { return }
    let p = point ?? CGPoint(x: bounds.midX, y: bounds.midY)
    let d = document(p)
    zoom = min(16, max(0.1, value))
    offset = CGPoint(x: p.x - d.x * zoom, y: p.y - d.y * zoom)
    owner.refresh()
  }
  override func magnify(with event: NSEvent) {
    setZoom(zoom * (1 + event.magnification), at: point(event))
  }
  override func keyDown(with event: NSEvent) {
    switch event.charactersIgnoringModifiers?.lowercased() {
    case " ": if gesture == 0 { space = true }
    case "p":
      owner.tool.selectedSegment = 0
      owner.changeTool(nil)
    case "v":
      owner.tool.selectedSegment = 1
      owner.changeTool(nil)
    case "\u{7f}", "\u{f728}": owner.deleteStroke(nil)
    case "\u{1b}": escape()
    default: super.keyDown(with: event)
    }
  }
  override func keyUp(with event: NSEvent) {
    if event.charactersIgnoringModifiers == " " { space = false } else { super.keyUp(with: event) }
  }
  override func resignFirstResponder() -> Bool {
    cancel()
    return super.resignFirstResponder()
  }
}

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate, NSMenuItemValidation {
  let editor = Editor()
  var window: NSWindow!
  var canvas: Canvas!
  let tool = NSSegmentedControl(
    labels: ["Pen", "Select"], trackingMode: .selectOne, target: nil, action: nil)
  let color = NSColorWell()
  let width = NSTextField(string: "6")
  let stepper = NSStepper()
  let scale = NSTextField(string: "100")
  let apply = NSButton(title: "Apply", target: nil, action: nil)
  let status = NSTextField(labelWithString: "")
  var url: URL?
  var approvedClose = false
  func applicationDidFinishLaunching(_ notification: Notification) {
    makeMenu()
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
    window.title = "Untitled — Native Sketch"
    window.minSize = NSSize(width: 640, height: 420)
    window.isReleasedWhenClosed = false
    window.delegate = self
    canvas = Canvas(owner: self)
    tool.selectedSegment = 0
    tool.target = self
    tool.action = #selector(changeTool(_:))
    tool.setAccessibilityLabel("Drawing tool")
    color.color = .black
    color.setAccessibilityLabel("Stroke color")
    color.widthAnchor.constraint(equalToConstant: 44).isActive = true
    width.setAccessibilityLabel("Stroke width in document points")
    width.target = self
    width.action = #selector(widthChanged(_:))
    width.widthAnchor.constraint(equalToConstant: 48).isActive = true
    let formatter = NumberFormatter()
    formatter.minimum = 0.1
    formatter.maximum = 256
    formatter.maximumFractionDigits = 1
    width.formatter = formatter
    stepper.minValue = 0.1
    stepper.maxValue = 256
    stepper.increment = 1
    stepper.doubleValue = 6
    stepper.target = self
    stepper.action = #selector(widthStepped(_:))
    stepper.setAccessibilityLabel("Adjust stroke width")
    scale.setAccessibilityLabel("Selection scale percent")
    scale.widthAnchor.constraint(equalToConstant: 56).isActive = true
    scale.target = self
    scale.action = #selector(resize(_:))
    apply.target = self
    apply.action = #selector(resize(_:))
    let controls = NSStackView(views: [
      tool, color, NSTextField(labelWithString: "Width"), width, stepper,
      NSTextField(labelWithString: "Scale %"), scale, apply,
    ])
    controls.spacing = 8
    controls.edgeInsets = NSEdgeInsets(top: 10, left: 12, bottom: 10, right: 12)
    status.font = .systemFont(ofSize: 12)
    status.textColor = .secondaryLabelColor
    status.setAccessibilityLabel("Document status")
    let root = NSView()
    window.contentView = root
    for v in [controls, canvas!, status] {
      v.translatesAutoresizingMaskIntoConstraints = false
      root.addSubview(v)
    }
    NSLayoutConstraint.activate([
      controls.topAnchor.constraint(equalTo: root.topAnchor),
      controls.leadingAnchor.constraint(equalTo: root.leadingAnchor),
      controls.trailingAnchor.constraint(lessThanOrEqualTo: root.trailingAnchor),
      canvas.topAnchor.constraint(equalTo: controls.bottomAnchor),
      canvas.leadingAnchor.constraint(equalTo: root.leadingAnchor),
      canvas.trailingAnchor.constraint(equalTo: root.trailingAnchor),
      canvas.bottomAnchor.constraint(equalTo: status.topAnchor, constant: -6),
      status.leadingAnchor.constraint(equalTo: root.leadingAnchor, constant: 12),
      status.trailingAnchor.constraint(equalTo: root.trailingAnchor, constant: -12),
      status.bottomAnchor.constraint(equalTo: root.bottomAnchor, constant: -8),
    ])
    let focusOrder: [NSView] = [tool, color, width, stepper, scale, apply, canvas]
    for index in focusOrder.indices {
      focusOrder[index].nextKeyView = focusOrder[(index + 1) % focusOrder.count]
    }
    refresh()
    window.center()
    window.makeKeyAndOrderFront(nil)
    window.makeFirstResponder(canvas)
    NSApp.activate(ignoringOtherApps: true)
    if CommandLine.arguments.contains("--native-smoke-test") {
      DispatchQueue.main.async { self.nativeSmoke() }
    }
  }
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
      item.target = self
    }
    add(menu("Native Sketch"), "Quit Native Sketch", #selector(quit(_:)), "q")
    let file = menu("File")
    add(file, "New", #selector(newDocument(_:)), "n")
    add(file, "Open…", #selector(openDocument(_:)), "o")
    add(file, "Save", #selector(saveDocument(_:)), "s")
    add(file, "Save As…", #selector(saveAs(_:)), "s", [.command, .shift])
    add(file, "Close", #selector(closeWindow(_:)), "w")
    let edit = menu("Edit")
    add(edit, "Undo", #selector(undo(_:)), "z")
    add(edit, "Redo", #selector(redo(_:)), "z", [.command, .shift])
    edit.addItem(.separator())
    for (name, action, key) in [
      ("Cut", #selector(NSText.cut(_:)), "x"), ("Copy", #selector(NSText.copy(_:)), "c"),
      ("Paste", #selector(NSText.paste(_:)), "v"),
      ("Select All", #selector(NSText.selectAll(_:)), "a"),
    ] { edit.addItem(withTitle: name, action: action, keyEquivalent: key) }
    add(edit, "Select Next Stroke", #selector(nextStroke(_:)), "]")
    add(edit, "Select Previous Stroke", #selector(previousStroke(_:)), "[")
    add(edit, "Delete Stroke", #selector(deleteStroke(_:)))
    let tools = menu("Tools")
    add(tools, "Pen", #selector(penTool(_:)))
    add(tools, "Select", #selector(selectTool(_:)))
    let view = menu("View")
    add(view, "Zoom In", #selector(zoomIn(_:)), "+")
    add(view, "Zoom Out", #selector(zoomOut(_:)), "-")
    add(view, "Actual Size", #selector(actualSize(_:)), "0")
  }
  @discardableResult func check(_ code: UInt32) -> Bool {
    if code == 0 { return true }
    let a = NSAlert()
    a.messageText = "The operation could not be completed"
    a.informativeText = "Engine error \(code). Your previous committed document is retained."
    a.runModal()
    return false
  }
  func checkFile(_ code: UInt32, url: URL, verb: String) -> Bool {
    guard code != 0 else { return true }
    let alert = NSAlert()
    alert.messageText = "Could not \(verb) \(url.lastPathComponent)"
    let reason =
      code == 5
      ? "The file is not a supported Native Sketch document, or its contents are invalid."
      : "Check that the folder exists and that you have permission to access it. The disk may be full or unavailable."
    alert.informativeText = "\(reason)\n\(url.path)\nYour current document is retained."
    alert.runModal()
    return false
  }
  func refresh() {
    guard canvas != nil else { return }
    let s = editor.state
    window.isDocumentEdited = s.dirty != 0
    window.title = "\(url?.lastPathComponent ?? "Untitled") — Native Sketch"
    let selected = s.selected_id != 0
    scale.isEnabled = selected
    apply.isEnabled = selected
    status.stringValue =
      "\(s.stroke_count) stroke\(s.stroke_count == 1 ? "" : "s") · \(selected ? "Stroke selected" : tool.selectedSegment == 0 ? "Pen — drag to draw" : "Select — click a stroke") · \(Int(canvas.zoom*100))% · Space-drag to pan"
    canvas.setAccessibilityValue(
      "\(s.stroke_count) strokes, \(selected ? "one selected" : "none selected"), zoom \(Int(canvas.zoom*100)) percent"
    )
    canvas.needsDisplay = true
  }
  @objc func penTool(_ sender: Any?) {
    tool.selectedSegment = 0
    changeTool(sender)
    window.makeFirstResponder(canvas)
  }
  @objc func selectTool(_ sender: Any?) {
    tool.selectedSegment = 1
    changeTool(sender)
    window.makeFirstResponder(canvas)
  }
  @objc func changeTool(_ sender: Any?) {
    canvas.cancel()
    if tool.selectedSegment == 0 { _ = sketch_select_clear(editor.handle) }
    refresh()
  }
  @objc func widthChanged(_ sender: Any?) {
    width.doubleValue = min(256, max(0.1, width.doubleValue))
    stepper.doubleValue = width.doubleValue
  }
  @objc func widthStepped(_ sender: Any?) { width.doubleValue = stepper.doubleValue }
  @objc func resize(_ sender: Any?) {
    canvas.cancel()
    let factor = scale.doubleValue / 100
    guard factor.isFinite && factor > 0 && factor <= 100 else {
      NSSound.beep()
      return
    }
    let b = editor.state.selection
    if check(sketch_resize_begin(editor.handle))
      && check(sketch_resize_update(editor.handle, factor, b.x + b.width / 2, b.y + b.height / 2))
    {
      _ = check(sketch_resize_commit(editor.handle))
    }
    if editor.state.interaction != 0 { _ = sketch_cancel(editor.handle) }
    scale.stringValue = "100"
    refresh()
  }
  @objc func nextStroke(_ sender: Any?) {
    canvas.cancel()
    tool.selectedSegment = 1
    _ = check(sketch_select_cycle(editor.handle, 1))
    refresh()
  }
  @objc func previousStroke(_ sender: Any?) {
    canvas.cancel()
    tool.selectedSegment = 1
    _ = check(sketch_select_cycle(editor.handle, -1))
    refresh()
  }
  @objc func deleteStroke(_ sender: Any?) {
    canvas.cancel()
    _ = check(sketch_delete_selected(editor.handle))
    refresh()
  }
  @objc func undo(_ sender: Any?) {
    if let text = window.firstResponder as? NSTextView {
      text.undoManager?.undo()
      return
    }
    canvas.cancel()
    _ = check(sketch_undo(editor.handle))
    refresh()
  }
  @objc func redo(_ sender: Any?) {
    if let text = window.firstResponder as? NSTextView {
      text.undoManager?.redo()
      return
    }
    canvas.cancel()
    _ = check(sketch_redo(editor.handle))
    refresh()
  }
  @objc func zoomIn(_ sender: Any?) { canvas.setZoom(canvas.zoom * 1.25) }
  @objc func zoomOut(_ sender: Any?) { canvas.setZoom(canvas.zoom / 1.25) }
  @objc func actualSize(_ sender: Any?) { canvas.setZoom(1) }
  func save(forcePanel: Bool = false) -> Bool {
    canvas.cancel()
    window.makeFirstResponder(canvas)
    var destination = url
    if destination == nil || forcePanel {
      let panel = NSSavePanel()
      panel.nameFieldStringValue = url?.lastPathComponent ?? "Untitled.nsketch"
      guard panel.runModal() == .OK else { return false }
      destination = panel.url
    }
    guard let destination else { return false }
    let success = checkFile(editor.file(destination, save: true), url: destination, verb: "save")
    if success { url = destination }
    refresh()
    return success
  }
  func mayReplace() -> Bool {
    canvas.cancel()
    guard editor.state.dirty != 0 else { return true }
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
  @objc func newDocument(_ sender: Any?) {
    guard mayReplace() else { return }
    if check(sketch_document_new(editor.handle)) {
      url = nil
      canvas.zoom = 1
      canvas.offset = CGPoint(x: 80, y: 80)
    }
    refresh()
  }
  @objc func openDocument(_ sender: Any?) {
    guard mayReplace() else { return }
    let panel = NSOpenPanel()
    panel.allowsMultipleSelection = false
    guard panel.runModal() == .OK, let destination = panel.url else { return }
    if checkFile(editor.file(destination, save: false), url: destination, verb: "open") {
      url = destination
      canvas.zoom = 1
      canvas.offset = CGPoint(x: 80, y: 80)
    }
    refresh()
  }
  @objc func quit(_ sender: Any?) { NSApp.terminate(sender) }
  @objc func closeWindow(_ sender: Any?) { window.performClose(sender) }
  func windowShouldClose(_ sender: NSWindow) -> Bool {
    approvedClose = mayReplace()
    return approvedClose
  }
  func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
    (approvedClose || mayReplace()) ? .terminateNow : .terminateCancel
  }
  func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }
  func applicationDidResignActive(_ notification: Notification) { canvas?.cancel() }
  func windowDidResignKey(_ notification: Notification) { canvas?.cancel() }
  func validateMenuItem(_ item: NSMenuItem) -> Bool {
    let s = editor.state
    if item.action == #selector(undo(_:)) {
      return (window?.firstResponder as? NSTextView)?.undoManager?.canUndo ?? (s.can_undo != 0)
    }
    if item.action == #selector(redo(_:)) {
      return (window?.firstResponder as? NSTextView)?.undoManager?.canRedo ?? (s.can_redo != 0)
    }
    if item.action == #selector(deleteStroke(_:)) {
      return s.selected_id != 0 && !(window?.firstResponder is NSTextView)
    }
    if item.action == #selector(nextStroke(_:)) || item.action == #selector(previousStroke(_:)) {
      return s.stroke_count > 0
    }
    return true
  }
  func nativeSmoke() {
    func event(_ type: NSEvent.EventType, _ x: Double, _ y: Double, _ time: Double) -> NSEvent {
      NSEvent.mouseEvent(
        with: type, location: canvas.convert(CGPoint(x: x, y: y), to: nil), modifierFlags: [],
        timestamp: time, windowNumber: window.windowNumber, context: nil, eventNumber: 1,
        clickCount: 1, pressure: 0)!
    }
    canvas.mouseDown(with: event(.leftMouseDown, 120, 120, 1))
    canvas.mouseDragged(with: event(.leftMouseDragged, 180, 160, 2))
    canvas.mouseUp(with: event(.leftMouseUp, 220, 180, 3))
    precondition(editor.state.stroke_count == 1)
    canvas.mouseDown(with: event(.leftMouseDown, 260, 220, 4))
    canvas.cancel()
    canvas.mouseUp(with: event(.leftMouseUp, 280, 240, 5))
    precondition(editor.state.stroke_count == 1)
    nextStroke(nil)
    scale.stringValue = "150"
    resize(nil)
    undo(nil)
    redo(nil)
    func geometry() -> [Double] {
      editor.commands().flatMap {
        [$0.x1, $0.y1, $0.x2, $0.y2, $0.x3, $0.y3, $0.red, $0.green, $0.blue, $0.alpha]
      }
    }
    let before = geometry()
    let point = CGPoint(x: 42, y: 37)
    canvas.setZoom(2)
    window.setContentSize(NSSize(width: 700, height: 480))
    precondition(geometry() == before)
    let roundtrip = canvas.document(canvas.screen(point))
    precondition(abs(roundtrip.x - point.x) < 1e-10 && abs(roundtrip.y - point.y) < 1e-10)
    print(
      "PASS: native AppKit mouse replay, cancellation, keyboard-equivalent selection/resize/history, viewport/window resize. Synthetic mouse events; no physical tablet."
    )
    _ = sketch_document_new(editor.handle)
    NSApp.terminate(nil)
  }
}

@main enum NativeSketch {
  @MainActor static func main() {
    guard sketch_abi_version() == 2 else { fatalError("Native engine ABI mismatch") }
    if CommandLine.arguments.contains("--smoke-test")
      || CommandLine.arguments.contains("--integration-test")
    {
      Integration.run()
      return
    }
    if CommandLine.arguments.contains("--timing") {
      Integration.timing()
      return
    }
    let app = NSApplication.shared
    app.setActivationPolicy(.regular)
    let delegate = AppDelegate()
    app.delegate = delegate
    withExtendedLifetime(delegate) { app.run() }
  }
}
