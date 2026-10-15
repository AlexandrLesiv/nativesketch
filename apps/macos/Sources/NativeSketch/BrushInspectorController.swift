import AppKit
import CSketchFFI

@MainActor final class BrushInspectorController: NSObject {
  unowned let owner: EditorWindowController
  init(owner: EditorWindowController) { self.owner = owner }
  let tool = NSSegmentedControl(
    labels: ["Pen", "Select"], trackingMode: .selectOne, target: nil, action: nil)
  let color = NSColorWell()
  let width = NSTextField(string: "6")
  let stepper = NSStepper()
  let scale = NSTextField(string: "100")
  let apply = NSButton(title: "Apply", target: nil, action: nil)
  var toolbarControls: NSStackView!
  let inspector = NSPopover()
  func configure() {
    tool.selectedSegment = 0
    tool.target = owner.actions
    tool.action = #selector(EditorActions.changeTool(_:))
    tool.setAccessibilityLabel("Drawing tool")
    color.color = .black
    NativeAccessibility.label(color, id: "sketch.brush.color", text: "Stroke color")
    color.widthAnchor.constraint(equalToConstant: 44).isActive = true
    NativeAccessibility.label(
      width, id: "sketch.brush.width", text: "Stroke width in document points")
    width.target = self
    width.action = #selector(BrushInspectorController.widthChanged(_:))
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
    stepper.action = #selector(BrushInspectorController.widthStepped(_:))
    stepper.setAccessibilityLabel("Adjust stroke width")
    scale.setAccessibilityLabel("Selection scale percent")
    scale.widthAnchor.constraint(equalToConstant: 56).isActive = true
    scale.target = owner.actions
    scale.action = #selector(EditorActions.resize(_:))
    apply.target = owner.actions
    apply.action = #selector(EditorActions.resize(_:))
    let brushRow = NSStackView(views: [
      color, NSTextField(labelWithString: "Width"), width, stepper,
    ])
    brushRow.spacing = 8
    let scaleRow = NSStackView(views: [NSTextField(labelWithString: "Scale %"), scale, apply])
    scaleRow.spacing = 8
    let controls = NSStackView(views: [brushRow, scaleRow])
    controls.orientation = .vertical
    controls.alignment = .leading
    controls.spacing = 12
    controls.edgeInsets = NSEdgeInsets(top: 12, left: 12, bottom: 12, right: 12)
    toolbarControls = controls
    let inspectorController = NSViewController()
    inspectorController.view = controls
    inspector.contentViewController = inspectorController
    inspector.contentSize = controls.fittingSize
    inspector.behavior = .transient
  }
  @objc func showInspector(_ sender: Any?) {
    owner.canvas.cancel()
    if inspector.isShown {
      inspector.close()
      return
    }
    inspector.show(relativeTo: tool.bounds, of: tool, preferredEdge: .maxY)
  }
  @objc func widthChanged(_ sender: Any?) {
    width.doubleValue = min(256, max(0.1, width.doubleValue))
    stepper.doubleValue = width.doubleValue
  }
  @objc func widthStepped(_ sender: Any?) { width.doubleValue = stepper.doubleValue }
}
