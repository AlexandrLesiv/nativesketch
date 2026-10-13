import AppKit
import CSketchFFI

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var window: NSWindow?

    func applicationDidFinishLaunching(_ notification: Notification) {
        let mainMenu = NSMenu()
        let appItem = NSMenuItem()
        let appMenu = NSMenu()
        appMenu.addItem(withTitle: "Quit Native Sketch", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        appItem.submenu = appMenu
        mainMenu.addItem(appItem)
        NSApp.mainMenu = mainMenu

        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 1000, height: 700),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )
        window.title = "Native Sketch"
        window.minSize = NSSize(width: 480, height: 360)
        window.isReleasedWhenClosed = false

        let title = NSTextField(labelWithString: "Native Sketch")
        title.font = .systemFont(ofSize: 24, weight: .semibold)
        let subtitle = NSTextField(labelWithString: "Drawing tools are the next milestone.")
        subtitle.textColor = .secondaryLabelColor
        let stack = NSStackView(views: [title, subtitle])
        stack.orientation = .vertical
        stack.spacing = 12
        stack.translatesAutoresizingMaskIntoConstraints = false

        let content = NSView()
        content.addSubview(stack)
        NSLayoutConstraint.activate([
            stack.centerXAnchor.constraint(equalTo: content.centerXAnchor),
            stack.centerYAnchor.constraint(equalTo: content.centerYAnchor),
        ])
        window.contentView = content
        window.center()
        window.makeKeyAndOrderFront(nil)
        self.window = window
        NSApp.activate(ignoringOtherApps: true)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }
}

@main
enum NativeSketch {
    @MainActor
    static func main() {
        guard sketch_abi_version() == 1 else {
            fatalError("The native engine interface does not match this application.")
        }
        if CommandLine.arguments.contains("--smoke-test") {
            print("PASS: Swift called the linked Rust ABI successfully.")
            return
        }
        let app = NSApplication.shared
        app.setActivationPolicy(.regular)
        let delegate = AppDelegate()
        app.delegate = delegate
        withExtendedLifetime(delegate) {
            app.run()
        }
    }
}

