import CSketchFFI
import UIKit
import UniformTypeIdentifiers

// The entire adapter is main-actor isolated. Rust owns document/edit/history semantics.
@MainActor final class MobileEditor {
  var handle: OpaquePointer?
  var failure: ((UInt32) -> Void)?
  init() {
    precondition(sketch_abi_version() == 2)
    reset()
  }
  isolated deinit { if let handle { sketch_editor_free(handle) } }
  func reset() {
    if let handle { sketch_editor_free(handle) }
    handle = nil
    precondition(sketch_editor_create(&handle) == 0)
  }
  @discardableResult func check(_ code: UInt32) -> Bool {
    if code == 6, let old = handle {
      handle = nil
      sketch_editor_free(old)
    }
    if code != 0 { failure?(code) }
    return code == 0
  }
  var state: SketchState {
    var s = SketchState()
    if handle != nil { _ = check(sketch_editor_state(handle, &s)) }
    return s
  }
  func commands() -> [SketchPathCommand] {
    guard handle != nil else { return [] }
    var count = 0
    guard check(sketch_render_copy(handle, nil, 0, &count)) else { return [] }
    var result = Array(repeating: SketchPathCommand(), count: count)
    return result.withUnsafeMutableBufferPointer {
      check(sketch_render_copy(handle, $0.baseAddress, $0.count, &count))
    } ? result : []
  }
  func snapshot() -> (Data, UInt64)? {
    var count = 0
    var revision: UInt64 = 0
    guard check(sketch_document_export(handle, nil, 0, &count, &revision)) else { return nil }
    var data = Data(count: count)
    guard
      data.withUnsafeMutableBytes({
        check(
          sketch_document_export(
            handle, $0.bindMemory(to: UInt8.self).baseAddress, count, &count, &revision))
      })
    else { return nil }
    return (data, revision)
  }
  func load(_ data: Data) -> Bool {
    data.withUnsafeBytes {
      check(sketch_document_import(handle, $0.bindMemory(to: UInt8.self).baseAddress, $0.count))
    }
  }
}

@MainActor final class DrawingView: UIView {
  unowned let owner: DrawingController
  var zoom = 1.0
  var offset = CGPoint(x: 32, y: 32)
  var mode = 0  // idle, ink, resize. Navigation is view state, never document state.
  var sampleID: UInt64 = 0
  var lastTimestamp = -Double.infinity
  var anchor = CGPoint.zero
  var initialVector = CGPoint.zero
  var ownerTouch: UITouch?
  var contacts = Set<UITouch>()
  var navigationTouches: [UITouch] = []
  var navigating = false
  var previousNavigation: (CGPoint, Double)?
  var oldSize = CGSize.zero
  init(owner: DrawingController) {
    self.owner = owner
    super.init(frame: .zero)
    backgroundColor = .white
    isMultipleTouchEnabled = true
    isAccessibilityElement = true
    accessibilityLabel = "Drawing canvas"
    accessibilityHint =
      "Use Select and Next Stroke to select without drawing. Two fingers pan and zoom."
    accessibilityCustomActions = [
      UIAccessibilityCustomAction(
        name: "Next stroke", target: owner, selector: #selector(DrawingController.nextStroke)),
      UIAccessibilityCustomAction(
        name: "Previous stroke", target: owner,
        selector: #selector(DrawingController.previousStroke)),
    ]
  }
  required init?(coder: NSCoder) { fatalError() }
  override var canBecomeFirstResponder: Bool { true }
  var editor: MobileEditor { owner.editor }
  func document(_ p: CGPoint) -> CGPoint {
    CGPoint(x: (p.x - offset.x) / zoom, y: (p.y - offset.y) / zoom)
  }
  func screen(_ p: CGPoint) -> CGPoint {
    CGPoint(x: p.x * zoom + offset.x, y: p.y * zoom + offset.y)
  }
  func cancel() {
    if editor.handle != nil { _ = editor.check(sketch_cancel(editor.handle)) }
    mode = 0
    ownerTouch = nil
    setNeedsDisplay()
  }
  func cancelAll() {
    cancel()
    contacts.removeAll()
    navigationTouches.removeAll()
    navigating = false
    previousNavigation = nil
    owner.refresh()
  }
  override func layoutSubviews() {
    super.layoutSubviews()
    if oldSize != .zero && oldSize != bounds.size {
      cancelAll()
      offset.x += (bounds.width - oldSize.width) / 2
      offset.y += (bounds.height - oldSize.height) / 2
    }
    oldSize = bounds.size
    contentScaleFactor = window?.screen.scale ?? traitCollection.displayScale
    setNeedsDisplay()
  }
  override func didMoveToWindow() {
    super.didMoveToWindow()
    if window == nil { cancelAll() }
    contentScaleFactor = window?.screen.scale ?? traitCollection.displayScale
  }
  func corners(_ b: SketchBounds) -> [CGPoint] {
    [
      CGPoint(x: b.x, y: b.y), CGPoint(x: b.x + b.width, y: b.y),
      CGPoint(x: b.x + b.width, y: b.y + b.height), CGPoint(x: b.x, y: b.y + b.height),
    ]
  }
  override func draw(_ rect: CGRect) {
    guard let c = UIGraphicsGetCurrentContext() else { return }
    c.setFillColor(UIColor.white.cgColor)
    c.fill(bounds)
    c.saveGState()
    c.translateBy(x: offset.x, y: offset.y)
    c.scaleBy(x: zoom, y: zoom)
    var id: UInt64?
    var path = CGMutablePath()
    var color = UIColor.black.cgColor
    func flush() {
      c.addPath(path)
      c.setFillColor(color)
      c.fillPath()
    }
    for command in editor.commands() {
      if id != command.stroke_id {
        if id != nil { flush() }
        id = command.stroke_id
        path = CGMutablePath()
        color =
          UIColor(red: command.red, green: command.green, blue: command.blue, alpha: command.alpha)
          .cgColor
      }
      switch command.verb {
      case 0: path.move(to: CGPoint(x: command.x1, y: command.y1))
      case 1: path.addLine(to: CGPoint(x: command.x1, y: command.y1))
      case 2:
        path.addCurve(
          to: CGPoint(x: command.x3, y: command.y3),
          control1: CGPoint(x: command.x1, y: command.y1),
          control2: CGPoint(x: command.x2, y: command.y2))
      default: path.closeSubpath()
      }
    }
    if id != nil { flush() }
    c.restoreGState()
    let s = editor.state
    if s.selected_id != 0 {
      let p = screen(CGPoint(x: s.selection.x, y: s.selection.y))
      c.setStrokeColor(UIColor.systemBlue.cgColor)
      c.setLineWidth(2)
      c.stroke(
        CGRect(x: p.x, y: p.y, width: s.selection.width * zoom, height: s.selection.height * zoom))
      for point in corners(s.selection) {
        let p = screen(point)
        let r = CGRect(x: p.x - 6, y: p.y - 6, width: 12, height: 12)
        c.setFillColor(UIColor.white.cgColor)
        c.fillEllipse(in: r)
        c.strokeEllipse(in: r)
      }
    }
  }
  func begin(at p: CGPoint, timestamp: Double, pressure: Double?, pencil: Bool) {
    guard editor.handle != nil else { return }
    becomeFirstResponder()
    if owner.tool.selectedSegmentIndex == 0 {
      var r: CGFloat = 0
      var g: CGFloat = 0
      var b: CGFloat = 0
      var a: CGFloat = 0
      owner.ink.getRed(&r, green: &g, blue: &b, alpha: &a)
      if editor.check(
        sketch_stroke_begin(
          editor.handle, 1, pencil ? 1 : 2,
          SketchBrush(red: r, green: g, blue: b, alpha: a, width: owner.strokeWidth)))
      {
        mode = 1
        sampleID = 0
        lastTimestamp = -Double.infinity
        move(to: p, timestamp: timestamp, pressure: pressure)
      }
    } else {
      let state = editor.state
      if state.selected_id != 0,
        let index = corners(state.selection).firstIndex(where: {
          let s = screen($0)
          return hypot(s.x - p.x, s.y - p.y) <= 22
        })
      {
        let points = corners(state.selection)
        anchor = points[(index + 2) % 4]
        initialVector = CGPoint(x: points[index].x - anchor.x, y: points[index].y - anchor.y)
        if editor.check(sketch_resize_begin(editor.handle)) { mode = 2 }
      } else {
        let p = document(p)
        _ = editor.check(sketch_select_at(editor.handle, p.x, p.y, 12 / zoom))
      }
    }
    owner.refresh()
  }
  func move(to p: CGPoint, timestamp: Double, pressure: Double?) {
    let d = document(p)
    if mode == 1 {
      guard timestamp > lastTimestamp else { return }
      lastTimestamp = timestamp
      sampleID += 1
      var sample = SketchSample(
        x: d.x, y: d.y, timestamp: timestamp, pressure: pressure ?? 0, sample_id: sampleID,
        has_pressure: pressure == nil ? 0 : 1, kind: 0)
      if !editor.check(sketch_stroke_append(editor.handle, 1, &sample, 1)) { cancel() }
    } else if mode == 2 {
      let denominator = max(
        0.001, initialVector.x * initialVector.x + initialVector.y * initialVector.y)
      let scale = max(
        0.01,
        ((d.x - anchor.x) * initialVector.x + (d.y - anchor.y) * initialVector.y) / denominator)
      if !editor.check(sketch_resize_update(editor.handle, scale, anchor.x, anchor.y)) { cancel() }
    }
    setNeedsDisplay()
  }
  func end() {
    if mode == 1 { _ = editor.check(sketch_stroke_commit(editor.handle, 1)) }
    if mode == 2 { _ = editor.check(sketch_resize_commit(editor.handle)) }
    mode = 0
    ownerTouch = nil
    owner.refresh()
  }
  func pressure(_ t: UITouch) -> Double? {
    t.type == .pencil && t.maximumPossibleForce > 0 ? Double(t.force / t.maximumPossibleForce) : nil
  }
  func navigationPair() -> (CGPoint, Double)? {
    let points = navigationTouches.filter { contacts.contains($0) }.map { $0.location(in: self) }
    guard points.count >= 2 else { return nil }
    let a = points[0]
    let b = points[1]
    return (CGPoint(x: (a.x + b.x) / 2, y: (a.y + b.y) / 2), hypot(a.x - b.x, a.y - b.y))
  }
  func beginNavigation() {
    cancel()
    navigating = true
    owner.refresh()
  }
  override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
    contacts.formUnion(touches)
    if contacts.count > 1 {
      if !navigating {
        navigationTouches = Array(contacts.prefix(2))
        beginNavigation()
        previousNavigation = navigationPair()
      }
      return
    }
    guard !navigating, let t = touches.first else { return }
    ownerTouch = t
    begin(
      at: t.location(in: self), timestamp: t.timestamp, pressure: pressure(t),
      pencil: t.type == .pencil)
  }
  override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
    if navigating {
      if let current = navigationPair(), let previous = previousNavigation {
        let d = document(previous.0)
        if previous.1 > 1 { zoom = min(16, max(0.1, zoom * current.1 / previous.1)) }
        offset = CGPoint(x: current.0.x - d.x * zoom, y: current.0.y - d.y * zoom)
        owner.refresh()
      }
      previousNavigation = navigationPair()
      return
    }
    guard let t = ownerTouch, touches.contains(t) else { return }
    for sample in event?.coalescedTouches(for: t) ?? [t] {
      move(to: sample.location(in: self), timestamp: sample.timestamp, pressure: pressure(sample))
    }
  }
  override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
    if !navigating, let t = ownerTouch, touches.contains(t) {
      move(to: t.location(in: self), timestamp: t.timestamp, pressure: pressure(t))
      end()
    }
    contacts.subtract(touches)
    previousNavigation = navigationPair()
    if contacts.isEmpty {
      navigating = false
      previousNavigation = nil
      navigationTouches.removeAll()
    }
  }
  override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) { cancelAll() }
}

@MainActor
final class DrawingController: UIViewController, UIDocumentPickerDelegate,
  UIColorPickerViewControllerDelegate
{
  let editor = MobileEditor()
  lazy var canvas = DrawingView(owner: self)
  let tool = UISegmentedControl(items: ["Pen", "Select"])
  let status = UILabel()
  var ink = UIColor.black
  var strokeWidth = 6.0
  var name = "Untitled"
  var exportRevision: UInt64?
  var exportURL: URL?
  var afterSave: (() -> Void)?
  var undoButton: UIBarButtonItem!
  var redoButton: UIBarButtonItem!
  override func viewDidLoad() {
    super.viewDidLoad()
    view.backgroundColor = .systemBackground
    editor.failure = { [weak self] code in
      self?.error(
        code == 6
          ? "The drawing engine stopped. Use New or Open to recover. Unsaved changes are unavailable."
          : "The operation failed (engine error \(code)). Your committed drawing is retained.")
    }
    tool.selectedSegmentIndex = 0
    tool.addTarget(self, action: #selector(changeTool), for: .valueChanged)
    tool.accessibilityLabel = "Drawing tool"
    tool.widthAnchor.constraint(equalToConstant: 150).isActive = true
    tool.heightAnchor.constraint(equalToConstant: 44).isActive = true
    undoButton = UIBarButtonItem(
      image: UIImage(systemName: "arrow.uturn.backward"), style: .plain, target: self,
      action: #selector(undoEdit))
    undoButton.accessibilityLabel = "Undo"
    redoButton = UIBarButtonItem(
      image: UIImage(systemName: "arrow.uturn.forward"), style: .plain, target: self,
      action: #selector(redoEdit))
    redoButton.accessibilityLabel = "Redo"
    navigationItem.rightBarButtonItems = [redoButton, undoButton]
    navigationItem.leftBarButtonItem = UIBarButtonItem(
      title: "Document",
      menu: UIMenu(children: [
        UIAction(title: "New", image: UIImage(systemName: "doc")) { [weak self] _ in
          self?.newDocument()
        },
        UIAction(title: "Open…", image: UIImage(systemName: "folder")) { [weak self] _ in
          self?.openDocument()
        },
        UIAction(title: "Save to Files…", image: UIImage(systemName: "square.and.arrow.up")) {
          [weak self] _ in self?.saveDocument()
        },
      ]))
    let options = UIBarButtonItem(
      image: UIImage(systemName: "ellipsis.circle"),
      menu: UIMenu(children: [
        UIAction(title: "Pen Style…") { [weak self] _ in self?.style() },
        UIAction(title: "Selection…") { [weak self] _ in self?.selection() },
        UIMenu(
          title: "Zoom",
          children: [
            UIAction(title: "Zoom In") { [weak self] _ in self?.zoomIn() },
            UIAction(title: "Zoom Out") { [weak self] _ in self?.zoomOut() },
            UIAction(title: "Actual Size") { [weak self] _ in self?.actualSize() },
          ]),
      ]))
    options.accessibilityLabel = "Drawing options: style, selection, and zoom"
    toolbarItems = [UIBarButtonItem(customView: tool), .flexibleSpace(), options]
    navigationController?.setToolbarHidden(false, animated: false)
    status.font = .preferredFont(forTextStyle: .caption1)
    status.adjustsFontForContentSizeCategory = true
    status.numberOfLines = 0
    status.accessibilityLabel = "Document status"
    for child in [canvas, status] {
      child.translatesAutoresizingMaskIntoConstraints = false
      view.addSubview(child)
    }
    NSLayoutConstraint.activate([
      canvas.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor),
      canvas.leadingAnchor.constraint(equalTo: view.leadingAnchor),
      canvas.trailingAnchor.constraint(equalTo: view.trailingAnchor),
      canvas.bottomAnchor.constraint(equalTo: status.topAnchor, constant: -4),
      status.leadingAnchor.constraint(
        equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 12),
      status.trailingAnchor.constraint(
        equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -12),
      status.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor, constant: -4),
    ])
    NotificationCenter.default.addObserver(
      self, selector: #selector(inactive), name: UIApplication.willResignActiveNotification,
      object: nil)
    refresh()
  }
  override func viewDidAppear(_ animated: Bool) {
    super.viewDidAppear(animated)
    canvas.becomeFirstResponder()
    if ProcessInfo.processInfo.arguments.contains("--integration-test") { runReplay() }
  }
  override func viewWillTransition(
    to size: CGSize, with coordinator: any UIViewControllerTransitionCoordinator
  ) {
    canvas.cancelAll()
    super.viewWillTransition(to: size, with: coordinator)
  }
  @objc func inactive() { canvas.cancelAll() }
  func refresh() {
    guard isViewLoaded else { return }
    let s = editor.state
    title = name + (s.dirty != 0 ? " •" : "")
    undoButton.isEnabled = s.can_undo != 0
    redoButton.isEnabled = s.can_redo != 0
    tool.isEnabled = editor.handle != nil
    status.text =
      "\(s.stroke_count) strokes · \(s.selected_id == 0 ? "None selected" : "Stroke selected") · \(Int(canvas.zoom*100))%"
    canvas.accessibilityValue = status.text
    canvas.setNeedsDisplay()
  }
  func error(_ message: String) {
    DispatchQueue.main.async { [weak self] in
      guard let self, self.presentedViewController == nil else { return }
      let a = UIAlertController(
        title: "Could not complete operation", message: message, preferredStyle: .alert)
      a.addAction(UIAlertAction(title: "OK", style: .default))
      self.present(a, animated: true)
    }
  }
  @objc func changeTool() {
    canvas.cancelAll()
    if tool.selectedSegmentIndex == 0 { _ = editor.check(sketch_select_clear(editor.handle)) }
    refresh()
  }
  @objc func undoEdit() {
    canvas.cancelAll()
    _ = editor.check(sketch_undo(editor.handle))
    refresh()
  }
  @objc func redoEdit() {
    canvas.cancelAll()
    _ = editor.check(sketch_redo(editor.handle))
    refresh()
  }
  @objc func nextStroke() -> Bool {
    canvas.cancelAll()
    tool.selectedSegmentIndex = 1
    _ = editor.check(sketch_select_cycle(editor.handle, 1))
    refresh()
    return true
  }
  @objc func previousStroke() -> Bool {
    canvas.cancelAll()
    tool.selectedSegmentIndex = 1
    _ = editor.check(sketch_select_cycle(editor.handle, -1))
    refresh()
    return true
  }
  @objc func deleteStroke() {
    canvas.cancelAll()
    _ = editor.check(sketch_delete_selected(editor.handle))
    refresh()
  }
  @objc func escape() {
    canvas.cancelAll()
    if editor.handle != nil { _ = editor.check(sketch_select_clear(editor.handle)) }
    refresh()
  }
  func isEditingText(_ root: UIView) -> Bool {
    if root.isFirstResponder && (root is UITextField || root is UITextView) { return true }
    return root.subviews.contains { isEditingText($0) }
  }
  @objc func zoomIn() { setZoom(canvas.zoom * 1.25) }
  @objc func zoomOut() { setZoom(canvas.zoom / 1.25) }
  @objc func actualSize() { setZoom(1) }
  func setZoom(_ zoom: Double) {
    canvas.cancelAll()
    let p = CGPoint(x: canvas.bounds.midX, y: canvas.bounds.midY)
    let d = canvas.document(p)
    canvas.zoom = min(16, max(0.1, zoom))
    canvas.offset = CGPoint(x: p.x - d.x * canvas.zoom, y: p.y - d.y * canvas.zoom)
    refresh()
  }
  override var keyCommands: [UIKeyCommand]? {
    guard presentedViewController == nil, !isEditingText(view) else { return nil }
    return [
      UIKeyCommand(
        title: "Save", action: #selector(saveDocument), input: "s", modifierFlags: .command),
      UIKeyCommand(
        title: "Open", action: #selector(openDocument), input: "o", modifierFlags: .command),
      UIKeyCommand(
        title: "New", action: #selector(newDocument), input: "n", modifierFlags: .command),
      UIKeyCommand(title: "Undo", action: #selector(undoEdit), input: "z", modifierFlags: .command),
      UIKeyCommand(
        title: "Redo", action: #selector(redoEdit), input: "z", modifierFlags: [.command, .shift]),
      UIKeyCommand(
        title: "Next Stroke", action: #selector(nextStroke), input: "]", modifierFlags: .command),
      UIKeyCommand(
        title: "Previous Stroke", action: #selector(previousStroke), input: "[",
        modifierFlags: .command),
      UIKeyCommand(title: "Cancel", action: #selector(escape), input: UIKeyCommand.inputEscape),
      UIKeyCommand(title: "Delete Stroke", action: #selector(deleteStroke), input: "\u{8}"),
    ]
  }
  @objc func style() {
    canvas.cancelAll()
    let a = UIAlertController(
      title: "Pen style", message: "Width in document points (0.1–256)", preferredStyle: .alert)
    a.addTextField {
      $0.text = String(self.strokeWidth)
      $0.keyboardType = .decimalPad
      $0.accessibilityLabel = "Stroke width"
    }
    a.addAction(
      UIAlertAction(title: "Apply width", style: .default) { [weak self, weak a] _ in
        guard let self, let value = Double(a?.textFields?.first?.text ?? ""), value.isFinite,
          value >= 0.1, value <= 256
        else {
          self?.error("Enter a width between 0.1 and 256.")
          return
        }
        self.strokeWidth = value
      })
    a.addAction(
      UIAlertAction(title: "Choose color…", style: .default) { [weak self] _ in
        guard let self else { return }
        let picker = UIColorPickerViewController()
        picker.selectedColor = self.ink
        picker.delegate = self
        self.present(picker, animated: true)
      })
    a.addAction(UIAlertAction(title: "Cancel", style: .cancel))
    present(a, animated: true)
  }
  func colorPickerViewControllerDidSelectColor(_ viewController: UIColorPickerViewController) {
    ink = viewController.selectedColor
  }
  func applyScale(_ text: String) -> Bool {
    canvas.cancelAll()
    guard let percent = Double(text.trimmingCharacters(in: .whitespacesAndNewlines)),
      percent.isFinite, percent > 0, percent <= 10000
    else {
      error("Enter a scale greater than 0 and at most 10000 percent.")
      return false
    }
    let b = editor.state.selection
    let success =
      editor.check(sketch_resize_begin(editor.handle))
      && editor.check(
        sketch_resize_update(editor.handle, percent / 100, b.x + b.width / 2, b.y + b.height / 2))
      && editor.check(sketch_resize_commit(editor.handle))
    if !success { canvas.cancelAll() }
    refresh()
    return success
  }
  @objc func selection() {
    canvas.cancelAll()
    let a = UIAlertController(
      title: "Selection", message: "Scale selected stroke uniformly (%)", preferredStyle: .alert)
    a.addTextField {
      $0.text = "100"
      $0.keyboardType = .decimalPad
      $0.accessibilityLabel = "Selection scale percent"
    }
    let resize = UIAlertAction(title: "Apply scale", style: .default) { [weak self, weak a] _ in
      _ = self?.applyScale(a?.textFields?.first?.text ?? "")
    }
    resize.isEnabled = editor.state.selected_id != 0
    a.addAction(resize)
    let next = UIAlertAction(title: "Next stroke", style: .default) { [weak self] _ in
      _ = self?.nextStroke()
    }
    next.isEnabled = editor.state.stroke_count > 0
    a.addAction(next)
    let previous = UIAlertAction(title: "Previous stroke", style: .default) { [weak self] _ in
      _ = self?.previousStroke()
    }
    previous.isEnabled = editor.state.stroke_count > 0
    a.addAction(previous)
    let delete = UIAlertAction(title: "Delete stroke", style: .destructive) { [weak self] _ in
      self?.deleteStroke()
    }
    delete.isEnabled = editor.state.selected_id != 0
    a.addAction(delete)
    a.addAction(UIAlertAction(title: "Cancel", style: .cancel))
    present(a, animated: true)
  }
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
        canvas.zoom = 1
        canvas.offset = CGPoint(x: 32, y: 32)
      }
      refresh()
    } catch { self.error(error.localizedDescription) }
  }
  func runReplay() {
    editor.failure = nil
    canvas.begin(at: CGPoint(x: 50, y: 50), timestamp: 1, pressure: nil, pencil: false)
    canvas.move(to: CGPoint(x: 140, y: 110), timestamp: 2, pressure: nil)
    canvas.end()
    precondition(editor.state.stroke_count == 1)
    canvas.begin(at: CGPoint(x: 180, y: 160), timestamp: 3, pressure: 0.5, pencil: true)
    canvas.beginNavigation()
    precondition(canvas.navigating && canvas.mode == 0)
    canvas.cancelAll()
    canvas.end()
    precondition(editor.state.stroke_count == 1)
    _ = nextStroke()
    precondition(applyScale("150"))
    undoEdit()
    redoEdit()
    let fixture = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    try! Data(repeating: 42, count: 65).write(to: fixture)
    do {
      _ = try Self.readDocument(fixture, limit: 64)
      preconditionFailure("Oversize import accepted")
    } catch {}
    precondition((try! Self.readDocument(fixture, limit: 65)).count == 65)
    try? FileManager.default.removeItem(at: fixture)
    let snapshot = editor.snapshot()!
    precondition(editor.state.dirty != 0)
    precondition(!editor.load(Data("bad".utf8)))
    precondition(editor.state.stroke_count == 1)
    precondition(editor.check(sketch_document_mark_saved(editor.handle, snapshot.1)))
    precondition(editor.state.dirty == 0)
    precondition(editor.load(snapshot.0))
    precondition(editor.state.stroke_count == 1)
    let point = CGPoint(x: 45, y: 70)
    canvas.zoom = 2
    canvas.offset = CGPoint(x: 14, y: 34)
    let mapped = canvas.document(canvas.screen(point))
    precondition(abs(mapped.x - point.x) < 1e-9 && abs(mapped.y - point.y) < 1e-9)
    let previous = canvas.document(CGPoint(x: canvas.bounds.midX, y: canvas.bounds.midY))
    canvas.frame.size = CGSize(width: 320, height: 480)
    canvas.layoutSubviews()
    let center = canvas.document(CGPoint(x: canvas.bounds.midX, y: canvas.bounds.midY))
    precondition(abs(previous.x - center.x) < 1e-9 && abs(previous.y - center.y) < 1e-9)
    let text =
      "PASS UIKit adapter replay: ink, cancellation, selection, scale/history, invalid import retention, revision save acknowledgement, roundtrip and rotation center mapping. Synthetic scalar input; no physical touch/Pencil."
    print(text)
    try? text.write(
      to: FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        .appendingPathComponent("replay-result.txt"), atomically: true, encoding: .utf8)
    exit(0)
  }
}

@main @MainActor final class NativeSketchApplication: UIResponder, UIApplicationDelegate {
  func application(
    _ application: UIApplication, configurationForConnecting session: UISceneSession,
    options: UIScene.ConnectionOptions
  ) -> UISceneConfiguration {
    let configuration = UISceneConfiguration(name: "Drawing", sessionRole: session.role)
    configuration.delegateClass = DrawingSceneDelegate.self
    return configuration
  }
}

@MainActor final class DrawingSceneDelegate: UIResponder, UIWindowSceneDelegate {
  var window: UIWindow?
  func scene(
    _ scene: UIScene, willConnectTo session: UISceneSession, options: UIScene.ConnectionOptions
  ) {
    guard let scene = scene as? UIWindowScene else { return }
    let window = UIWindow(windowScene: scene)
    window.rootViewController = UINavigationController(rootViewController: DrawingController())
    self.window = window
    window.makeKeyAndVisible()
  }
  func sceneWillResignActive(_ scene: UIScene) {
    (window?.rootViewController as? UINavigationController)?.viewControllers.compactMap {
      $0 as? DrawingController
    }.forEach { $0.inactive() }
  }
  func sceneDidDisconnect(_ scene: UIScene) {
    sceneWillResignActive(scene)
    window = nil
  }
}
