import AppKit
import CoreGraphics
import Metal

@MainActor final class RenderBenchmarkVisibility {
  private let frame: CGRect
  private let canvasSize: CGSize
  private let display: CGDirectDisplayID
  private let scale: CGFloat
  init(owner: EditorWindowController) throws {
    guard let screen = owner.window.screen,
      let number = screen.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber
    else { throw RenderBenchmarkError(reason: "missing-display") }
    frame = owner.window.frame
    canvasSize = owner.canvas.bounds.size
    display = number.uint32Value
    scale = owner.window.backingScaleFactor
  }
  func validate(_ owner: EditorWindowController) throws {
    let window = owner.window!
    guard NSApp.isActive, window.isKeyWindow, window.isVisible, !window.isMiniaturized,
      window.occlusionState.contains(.visible)
    else {
      throw RenderBenchmarkError(reason: "window-not-active-visible")
    }
    guard window.frame == frame, owner.canvas.bounds.size == canvasSize,
      window.backingScaleFactor == scale,
      let screen = window.screen, screen.visibleFrame.contains(frame),
      (screen.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber)?.uint32Value
        == display
    else { throw RenderBenchmarkError(reason: "window-moved-resized-or-display-changed") }
    guard let session = CGSessionCopyCurrentDictionary() as? [String: Any],
      session[kCGSessionOnConsoleKey as String] as? Bool == true,
      session["CGSSessionScreenIsLocked"] as? Bool != true
    else { throw RenderBenchmarkError(reason: "session-locked-or-off-console") }
    let screenRect = window.convertToScreen(owner.canvas.convert(owner.canvas.bounds, to: nil))
    let top = NSScreen.screens.first?.frame.maxY ?? 0
    let canvasRect = CGRect(
      x: screenRect.minX, y: top - screenRect.maxY,
      width: screenRect.width, height: screenRect.height)
    guard
      let windows = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID)
        as? [[String: Any]]
    else { throw RenderBenchmarkError(reason: "window-list-unavailable") }
    var found = false
    for entry in windows {
      if (entry[kCGWindowNumber as String] as? NSNumber)?.intValue == window.windowNumber {
        found = true
        break
      }
      if (entry[kCGWindowOwnerPID as String] as? NSNumber)?.int32Value == getpid() { continue }
      if (entry[kCGWindowAlpha as String] as? NSNumber)?.doubleValue == 0 { continue }
      guard let bounds = entry[kCGWindowBounds as String] as? [String: Any],
        let rect = CGRect(dictionaryRepresentation: bounds as CFDictionary)
      else {
        throw RenderBenchmarkError(reason: "overlay-bounds-unavailable")
      }
      if rect.intersects(canvasRect) {
        throw RenderBenchmarkError(reason: "external-window-overlaps-canvas")
      }
    }
    if !found { throw RenderBenchmarkError(reason: "benchmark-window-not-listed") }
  }
  func metadata(owner: EditorWindowController) -> [String: String] {
    let mode = CGDisplayCopyDisplayMode(display)
    #if DEBUG
      let configuration = "debug"
    #else
      let configuration = "release"
    #endif
    return [
      "fixture": RenderBenchmarkFixture.identity,
      "input": "synthetic shared-command replay; not native pen events or physical latency",
      "cachePolicy": "production cached Skia; CPU cached/reference comparison is separate",
      "os": ProcessInfo.processInfo.operatingSystemVersionString,
      "gpu": MTLCreateSystemDefaultDevice()?.name ?? "unknown",
      "displayID": String(display), "displayName": owner.window.screen?.localizedName ?? "unknown",
      "refreshHz": String(mode?.refreshRate ?? 0), "scale": String(Double(scale)),
      "logicalWidth": String(Double(canvasSize.width)),
      "logicalHeight": String(Double(canvasSize.height)),
      "pixelWidth": String(Int(ceil(canvasSize.width * scale))),
      "pixelHeight": String(Int(ceil(canvasSize.height * scale))),
      "configuration": configuration,
      "initialZoom": String(owner.canvas.zoom),
      "initialOffsetX": String(Double(owner.canvas.offset.x)),
      "initialOffsetY": String(Double(owner.canvas.offset.y)),
      "visibility":
        "active key visible window, console unlocked, fixed bounds/display, window-list overlap check",
    ]
  }
}
