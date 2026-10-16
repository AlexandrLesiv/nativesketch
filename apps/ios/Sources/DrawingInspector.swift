import CSketchFFI
import UIKit
import UniformTypeIdentifiers

extension DrawingController {
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
}
