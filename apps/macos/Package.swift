// swift-tools-version: 6.0
import Foundation
import PackageDescription

let repoRoot = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .deletingLastPathComponent()
let rustProfile = ProcessInfo.processInfo.environment["SKETCH_RUST_PROFILE"] ?? "debug"

let package = Package(
    name: "NativeSketch",
    platforms: [.macOS(.v14)],
    products: [.executable(name: "NativeSketch", targets: ["NativeSketch"])],
    targets: [
        .systemLibrary(name: "CSketchFFI", path: "Sources/CSketchFFI"),
        .executableTarget(
            name: "NativeSketch",
            dependencies: ["CSketchFFI"],
            linkerSettings: [
                .unsafeFlags([repoRoot.appendingPathComponent("target/\(rustProfile)/libsketch_ffi.a").path]),
            ]
        ),
    ]
)

