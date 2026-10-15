// swift-tools-version: 6.0
import Foundation
import PackageDescription

let repoRoot = URL(fileURLWithPath: #filePath)
  .deletingLastPathComponent()
  .deletingLastPathComponent()
  .deletingLastPathComponent()
let rustProfile = ProcessInfo.processInfo.environment["SKETCH_RUST_PROFILE"] ?? "debug"

let testControls = ProcessInfo.processInfo.environment["SKETCH_NATIVE_TEST_CONTROLS"] == "1"
let nativeDependencies: [Target.Dependency] =
  testControls ? ["CSketchSession", "CSketchSessionTest"] : ["CSketchSession"]
let package = Package(
  name: "NativeSketch",
  platforms: [.macOS(.v14)],
  products: [.executable(name: "NativeSketch", targets: ["NativeSketch"])],
  targets: [
    .testTarget(name: "NativeSketchTests", dependencies: ["NativeSketch"]),
    .systemLibrary(name: "CSketchSession", path: "Sources/CSketchSession"),
    .systemLibrary(name: "CSketchSessionTest", path: "Sources/CSketchSessionTest"),
    .executableTarget(
      name: "NativeSketch",
      dependencies: nativeDependencies,
      swiftSettings: testControls
        ? [.define("NATIVE_TEST_CONTROLS", .when(configuration: .debug))] : [],
      linkerSettings: [
        .linkedLibrary("c++"),
        .linkedFramework("Metal"),
        .linkedFramework("MetalKit"),
        .linkedFramework("ApplicationServices"),
        .linkedFramework("QuartzCore"),
        .linkedFramework("CoreGraphics"),
        .linkedFramework("CoreText"),
        .unsafeFlags([
          repoRoot.appendingPathComponent("target/\(rustProfile)/libsketch_native_ffi.a").path
        ])
      ]
    ),
  ]
)
