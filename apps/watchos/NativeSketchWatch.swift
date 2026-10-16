import Darwin
import SwiftUI
import WatchConnectivity

@main struct NativeSketchWatch: App {
  var body: some Scene {
    WindowGroup {
      #if DEBUG
        if ProcessInfo.processInfo.arguments.contains("--preview-large-text") {
          WatchViewer().environment(\.dynamicTypeSize, .accessibility3)
        } else {
          WatchViewer()
        }
      #else
        WatchViewer()
      #endif
    }
  }
}
