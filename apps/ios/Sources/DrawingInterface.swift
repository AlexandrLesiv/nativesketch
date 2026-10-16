import CSketchFFI
import UIKit
import UniformTypeIdentifiers

extension DrawingController {
  func configureInterface() {
    view.backgroundColor = .systemBackground
    editor.failure = { [weak self] code in
      self?.error(
        code == 6
          ? "The drawing engine stopped. Use New or Open to recover. Unsaved changes are unavailable."
          : "The operation failed (engine error \(code)). Your committed drawing is retained.")
    }
    #if os(visionOS)
      tool.selectedSegmentIndex = 1
      canvas.accessibilityHint =
        "Select strokes or choose Pan to navigate. Choose Pen explicitly to draw. Use Drawing options to zoom."
    #else
      tool.selectedSegmentIndex = 0
    #endif
    tool.addTarget(self, action: #selector(changeTool), for: .valueChanged)
    tool.accessibilityLabel = "Drawing tool"
    #if os(visionOS)
      tool.widthAnchor.constraint(equalToConstant: 260).isActive = true
    #else
      tool.widthAnchor.constraint(equalToConstant: 150).isActive = true
    #endif
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
    var documentActions: [UIMenuElement] = [
      UIAction(title: "New", image: UIImage(systemName: "doc")) { [weak self] _ in
        self?.newDocument()
      },
      UIAction(title: "Open…", image: UIImage(systemName: "folder")) { [weak self] _ in
        self?.openDocument()
      },
      UIAction(title: "Save to Files…", image: UIImage(systemName: "square.and.arrow.up")) {
        [weak self] _ in self?.saveDocument()
      },
    ]
    #if !os(visionOS)
      if UIDevice.current.userInterfaceIdiom == .phone {
        watchSender.activate()
        documentActions.append(
          UIAction(title: "Apple Watch Transfer Status") { [weak self] _ in
            self?.showWatchStatus()
          })
        documentActions.append(
          UIAction(title: "Send Copy to Apple Watch", image: UIImage(systemName: "applewatch")) {
            [weak self] _ in
            self?.sendToWatch()
          })
      }
    #endif
    navigationItem.leftBarButtonItem = UIBarButtonItem(
      title: "Document", menu: UIMenu(children: documentActions))
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
            UIAction(title: "Fit Drawing") { [weak self] _ in self?.fitDrawing() },
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
}
