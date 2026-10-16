import CSketchFFI
import UIKit
import UniformTypeIdentifiers

@MainActor
final class DrawingController: UIViewController, UIDocumentPickerDelegate,
  UIColorPickerViewControllerDelegate
{
  let editor = MobileEditor()
  lazy var canvas = DrawingView(owner: self)
  #if os(visionOS)
    let tool = UISegmentedControl(items: ["Pen", "Select", "Pan"])
  #else
    let tool = UISegmentedControl(items: ["Pen", "Select"])
    lazy var watchSender = WatchCopySender()
    var lastWatchStatus = "No Apple Watch copy has been queued in this session."
  #endif
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
    configureInterface()
  }
  override func viewDidAppear(_ animated: Bool) {
    super.viewDidAppear(animated)
    canvas.becomeFirstResponder()
    #if DEBUG
      if ProcessInfo.processInfo.arguments.contains("--integration-test") { runReplay() }
    #endif
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
  func isEditingText(_ root: UIView) -> Bool {
    if root.isFirstResponder && (root is UITextField || root is UITextView) { return true }
    return root.subviews.contains { isEditingText($0) }
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

}
