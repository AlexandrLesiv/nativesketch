import AppKit
import QuartzCore
import Metal
import CoreVideo

// The same host code is linked in every size variant. Build-host records the exact
// Cargo selection; Rust validates it again before creating a GPU/session.
let availableBackends: [String] = {
    guard let url = Bundle.main.url(forResource: "build", withExtension: "json"),
          let data = try? Data(contentsOf: url),
          let metadata = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any],
          let backends = metadata["available_backends"] as? [String],
          [["skia", "vello"], ["skia"], ["vello"], ["baseline"]].contains(backends) else { return [] }
    return backends
}()
struct Options {
    var backend = availableBackends.first ?? "", fixture = "live", mode = "completion"
    var width = 1600, height = 1000, warmup = 120, frames = 600
    var repeatID = "0"
    var duration: Double? = nil
    var output = ""
    var interactive = true
    // Finder/Launch Services may launch with / as the working directory. Never derive
    // interactive result paths from it. Explicit CLI output remains authoritative.
    static func writableResultRoot(_ candidates: [URL]) throws -> URL {
        let files = FileManager.default
        for root in candidates {
            do {
                try files.createDirectory(at: root, withIntermediateDirectories: true)
                // A real probe also catches ACL/read-only-volume restrictions.
                let probe = root.appendingPathComponent(".write-probe-" + UUID().uuidString)
                try Data().write(to: probe, options: .withoutOverwriting)
                try files.removeItem(at: probe)
                return root
            } catch { continue }
        }
        throw HostError.message("No writable result directory. Relaunch with --output /absolute/writable/path")
    }
    static func defaultOutput() throws -> String {
        var candidates: [URL] = []
        if let resource = Bundle.main.url(forResource: "build", withExtension: "json"),
           let data = try? Data(contentsOf: resource),
           let metadata = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any],
           let root = metadata["result_root"] as? String, root.hasPrefix("/") {
            candidates.append(URL(fileURLWithPath: root, isDirectory: true))
        }
        if let caches = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first {
            candidates.append(caches.appendingPathComponent("org.nativesketch.renderer-comparison/results", isDirectory: true))
        }
        candidates.append(FileManager.default.temporaryDirectory.appendingPathComponent("org.nativesketch.renderer-comparison/results", isDirectory: true))
        return try writableResultRoot(candidates).appendingPathComponent("manual-" + UUID().uuidString, isDirectory: true).path
    }
    static func parse() throws -> Options {
        guard !availableBackends.isEmpty else { throw HostError.message("Missing or invalid build.json available_backends; rebuild with build-host") }
        var o = Options(); let a = Array(CommandLine.arguments.dropFirst()); o.interactive = a.isEmpty || a.contains("--interactive")
        var i = 0; var seen = Set<String>()
        while i < a.count {
            let key = a[i]; i += 1
            if key == "--interactive" { continue }
            guard i < a.count, seen.insert(key).inserted else { throw HostError.message("Missing or duplicate option: \(key)") }
            let value = a[i]; i += 1
            func integer() throws -> Int { guard let n = Int(value), n >= 0 else { throw HostError.message("Invalid integer for \(key)") }; return n }
            switch key {
            case "--backend": o.backend = value
            case "--fixture": o.fixture = value
            case "--mode": o.mode = value
            case "--width": o.width = try integer()
            case "--height": o.height = try integer()
            case "--warmup": o.warmup = try integer()
            case "--frames": o.frames = try integer()
            case "--duration-seconds": guard let n = Double(value), n.isFinite, n > 0 else { throw HostError.message("Invalid duration") }; o.duration = n
            case "--repeat-id": o.repeatID = value
            case "--output": o.output = URL(fileURLWithPath: value).standardizedFileURL.path
            default: throw HostError.message("Unknown option: \(key)")
            }
        }
        guard availableBackends.contains(o.backend) else { throw HostError.message("Backend \(o.backend) is not compiled; available: \(availableBackends.joined(separator: ", "))") }
        guard fixtures.contains(o.fixture), ["completion", "paced"].contains(o.mode), o.width > 0, o.height > 0, o.width <= 16384, o.height <= 16384, o.frames > 0, !(seen.contains("--frames") && o.duration != nil) else { throw HostError.message("Invalid backend, fixture, mode, dimensions, or frame/duration combination") }
        if o.output.isEmpty { o.output = try defaultOutput() }
        return o
    }
}
let fixtures = ["live", "dense100", "dense1000", "dense5000", "panzoom", "alpha", "clip", "image"]
enum HostError: Error { case message(String) }
final class StopState {
    private let lock = NSLock(); private var value: (String, String)?
    func request(_ status: String, _ reason: String) { lock.lock(); defer { lock.unlock() }; if value == nil { value = (status, reason) } }
    func read() -> (String, String)? { lock.lock(); defer { lock.unlock() }; return value }
}
final class Canvas: NSView {
    let metal = CAMetalLayer()
    var physicalSize = CGSize(width: 1600, height: 1000) { didSet { needsLayout = true } }
    override init(frame: NSRect) { super.init(frame: frame); wantsLayer = true; layer = CALayer(); layer?.backgroundColor = NSColor.windowBackgroundColor.cgColor; layer?.addSublayer(metal); metal.pixelFormat = .bgra8Unorm; metal.framebufferOnly = false; metal.maximumDrawableCount = 2; metal.colorspace = CGColorSpace(name: CGColorSpace.sRGB); setAccessibilityElement(true); setAccessibilityLabel("Deterministic renderer preview; no drawing input") }
    override func layout() {
        super.layout()
        let scale = min(bounds.width / physicalSize.width, bounds.height / physicalSize.height)
        let size = CGSize(width: physicalSize.width * scale, height: physicalSize.height * scale)
        CATransaction.begin(); CATransaction.setDisableActions(true)
        metal.frame = CGRect(x: (bounds.width - size.width) / 2, y: (bounds.height - size.height) / 2, width: size.width, height: size.height)
        CATransaction.commit()
    }
    required init?(coder: NSCoder) { fatalError() }
}
final class Host: NSObject, NSApplicationDelegate, NSWindowDelegate, NSMenuItemValidation {
    var options: Options
    var window: NSWindow!
    let canvas = Canvas(frame: .zero), status = NSTextField(wrappingLabelWithString: "Ready")
    let backend = NSPopUpButton(), fixture = NSPopUpButton(), resolution = NSPopUpButton(), mode = NSPopUpButton()
    let run = NSButton(title: "Run", target: nil, action: nil), stop = NSButton(title: "Stop", target: nil, action: nil), reveal = NSButton(title: "Reveal Results", target: nil, action: nil)
    let worker = DispatchQueue(label: "org.nativesketch.renderer-experiment", qos: .userInitiated)
    var active: StopState?, closing = false, exitCode: Int32 = 0
    var runNumber = 0
    let originalOutput: String
    // Main-thread-only lifecycle state; the worker only accesses the locked StopState.
    var workspaceObservers: [NSObjectProtocol] = []
    var activity: NSObjectProtocol?
    let activityOptions: ProcessInfo.ActivityOptions = [.userInitiated, .latencyCritical, .idleDisplaySleepDisabled]
    var lifecycleStart: [String: Any]?
    var lifecycleEvents: [[String: Any]] = []
    var sessionResigned = false, systemSleeping = false, screensSleeping = false
    var measuredElapsed = 0.0
    func visibleState() -> [String: Any] {
        precondition(Thread.isMainThread)
        let session = CGSessionCopyCurrentDictionary() as? [String: Any]
        let onConsole = (session?[kCGSessionOnConsoleKey as String] as? NSNumber)?.boolValue ?? false
        let screenID = (window.screen?.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber)?.uint32Value ?? CGMainDisplayID()
        return ["window_visible": window.isVisible, "window_unoccluded": window.occlusionState.contains(.visible),
                "window_miniaturized": window.isMiniaturized, "window_level": window.level.rawValue, "session_on_console": onConsole,
                "session_resigned": sessionResigned, "system_sleeping": systemSleeping,
                "screens_sleeping": screensSleeping, "display_asleep": CGDisplayIsAsleep(screenID) != 0,
                "system_uptime_seconds": ProcessInfo.processInfo.systemUptime]
    }
    func canMeasure(_ state: [String: Any]) -> Bool {
        ["window_visible", "window_unoccluded", "session_on_console"].allSatisfy { state[$0] as? Bool == true }
            && ["window_miniaturized", "session_resigned", "system_sleeping", "screens_sleeping", "display_asleep"].allSatisfy { state[$0] as? Bool == false }
    }
    func lifecycleEvent(_ name: String, invalidating: Bool) {
        precondition(Thread.isMainThread)
        guard active != nil else { return }
        if invalidating { invalidate(name) }
        else { lifecycleEvents.append(["event": name, "invalidating": false, "state": visibleState()]) }
    }
    @objc func workspaceEvent(_ notification: Notification) {
        precondition(Thread.isMainThread)
        switch notification.name {
        case NSWorkspace.sessionDidResignActiveNotification: sessionResigned = true
        case NSWorkspace.sessionDidBecomeActiveNotification: sessionResigned = false
        case NSWorkspace.willSleepNotification: systemSleeping = true
        case NSWorkspace.didWakeNotification: systemSleeping = false
        case NSWorkspace.screensDidSleepNotification: screensSleeping = true
        case NSWorkspace.screensDidWakeNotification: screensSleeping = false
        default: return
        }
        lifecycleEvent(notification.name.rawValue, invalidating: sessionResigned || systemSleeping || screensSleeping)
    }
    init(_ options: Options) { self.options = options; originalOutput = options.output }
    func validateMenuItem(_ menuItem: NSMenuItem) -> Bool {
        if menuItem.action == #selector(start) { return active == nil }
        if menuItem.action == #selector(cancel) { return active != nil }
        return true
    }
    func applicationDidFinishLaunching(_ notification: Notification) {
        for name in [NSWorkspace.sessionDidResignActiveNotification, NSWorkspace.sessionDidBecomeActiveNotification,
                     NSWorkspace.willSleepNotification, NSWorkspace.didWakeNotification,
                     NSWorkspace.screensDidSleepNotification, NSWorkspace.screensDidWakeNotification] {
            workspaceObservers.append(NSWorkspace.shared.notificationCenter.addObserver(forName: name, object: nil, queue: .main) { [weak self] notification in
                self?.workspaceEvent(notification)
            })
        }
        let menuBar = NSMenu()
        let applicationItem = NSMenuItem(); menuBar.addItem(applicationItem)
        let applicationMenu = NSMenu(title: "Renderer Comparison")
        let quitItem = NSMenuItem(title: "Quit Renderer Comparison", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        quitItem.target = NSApp; applicationMenu.addItem(quitItem); applicationItem.submenu = applicationMenu
        let experimentItem = NSMenuItem(); menuBar.addItem(experimentItem)
        let experimentMenu = NSMenu(title: "Experiment")
        let runItem = NSMenuItem(title: "Run", action: #selector(start), keyEquivalent: "r"); runItem.target = self
        let stopItem = NSMenuItem(title: "Stop", action: #selector(cancel), keyEquivalent: "."); stopItem.target = self
        experimentMenu.addItem(runItem); experimentMenu.addItem(stopItem); experimentItem.submenu = experimentMenu
        NSApp.mainMenu = menuBar
        window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 900, height: 680), styleMask: [.titled, .closable, .miniaturizable, .resizable], backing: .buffered, defer: false)
        // A controlled automatic trial must remain visible when the orchestration UI updates.
        // This local level is restored after output; ordinary interactive launches stay normal.
        if !options.interactive { window.level = .floating }
        window.title = "Native Sketch — Renderer Comparison"; window.delegate = self; window.minSize = NSSize(width: 640, height: 480)
        backend.addItems(withTitles: availableBackends); fixture.addItems(withTitles: fixtures); resolution.addItems(withTitles: ["1600 × 1000", "2560 × 1600"]); mode.addItems(withTitles: ["completion", "paced"])
        backend.selectItem(withTitle: options.backend); fixture.selectItem(withTitle: options.fixture); mode.selectItem(withTitle: options.mode)
        let sizeTitle = "\(options.width) × \(options.height)"; if resolution.item(withTitle: sizeTitle) == nil { resolution.addItem(withTitle: sizeTitle) }; resolution.selectItem(withTitle: sizeTitle)
        let grid = NSGridView(views: [[NSTextField(labelWithString: availableBackends == ["baseline"] ? "Blank size control" : "Backend"), backend, NSTextField(labelWithString: "Fixture"), fixture], [NSTextField(labelWithString: "Physical pixels"), resolution, NSTextField(labelWithString: "Mode"), mode]])
        // Keep labels next to their selectors at every window size. This bounded
        // native control group fits inside the 640-point minimum window's insets.
        grid.translatesAutoresizingMaskIntoConstraints = false
        grid.widthAnchor.constraint(equalToConstant: 560).isActive = true
        grid.columnSpacing = 12
        grid.column(at: 0).width = 100
        grid.column(at: 2).width = 55
        for (control, label) in [(backend, "Backend"), (fixture, "Fixture"), (resolution, "Physical pixel resolution"), (mode, "Execution mode")] { control.setAccessibilityLabel(label) }
        run.target = self; run.action = #selector(start); stop.target = self; stop.action = #selector(cancel); stop.keyEquivalent = "."; stop.keyEquivalentModifierMask = .command; stop.isEnabled = false
        reveal.target = self; reveal.action = #selector(revealResults)
        let buttons = NSStackView(views: [run, stop, reveal]); buttons.orientation = .horizontal
        let stack = NSStackView(views: [grid, buttons, status, canvas]); stack.orientation = .vertical; stack.alignment = .leading; stack.spacing = 10; stack.edgeInsets = NSEdgeInsets(top: 14, left: 14, bottom: 14, right: 14)
        window.contentView = stack; canvas.translatesAutoresizingMaskIntoConstraints = false
        NSLayoutConstraint.activate([canvas.widthAnchor.constraint(equalTo: stack.widthAnchor, constant: -28), canvas.heightAnchor.constraint(greaterThanOrEqualToConstant: 260), status.widthAnchor.constraint(equalTo: canvas.widthAnchor)])
        status.setAccessibilityLabel("Experiment status")
        window.center(); window.makeKeyAndOrderFront(nil); NSApp.activate(ignoringOtherApps: true)
        // Allow launch/activation to settle before any trial or initialization starts.
        // This delay is outside timing; every in-trial occlusion still invalidates.
        if !options.interactive { DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) { self.start() } }
    }
    func setStatus(_ value: String) { status.stringValue = value; NSAccessibility.post(element: status, notification: .valueChanged) }
    @objc func revealResults() { NSWorkspace.shared.selectFile(nil, inFileViewerRootedAtPath: options.output) }
    @objc func cancel() { if active != nil { lifecycleEvents.append(["event": "Stop requested", "invalidating": false, "state": visibleState()]) }; active?.request("cancelled", "Stop requested"); setStatus("Stopping safely; partial results will be retained…") }
    func invalidate(_ reason: String) { guard let active else { return }; lifecycleEvents.append(["event": reason, "invalidating": true, "state": visibleState()]); active.request("invalidated", reason); setStatus("Trial invalidated: \(reason). Finishing partial results…") }
    func windowDidChangeOcclusionState(_ n: Notification) {
        lifecycleEvent("Window occlusion changed", invalidating: !window.occlusionState.contains(.visible))
    }
    func windowWillStartLiveResize(_ n: Notification) { invalidate("Window resize") }
    func windowDidResize(_ n: Notification) { invalidate("Window dimensions changed") }
    func windowDidMiniaturize(_ n: Notification) { invalidate("Window minimized") }
    func windowDidChangeScreen(_ n: Notification) { invalidate("Display changed") }
    func windowDidChangeBackingProperties(_ n: Notification) { invalidate("Backing scale changed") }
    func windowShouldClose(_ sender: NSWindow) -> Bool { if active != nil { closing = true; cancel(); return false }; NSApp.terminate(nil); return true }
    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply { if active != nil { closing = true; cancel(); return .terminateCancel }; return .terminateNow }
    func applicationWillTerminate(_ notification: Notification) {
        // AppKit's terminate path exits the process without returning from app.run().
        // finishUI has already drained the renderer and written host-status before
        // requesting termination. Preserve its verdict for command-line runners.
        fflush(nil)
        exit(exitCode)
    }
    @objc func start() {
        guard active == nil else { return }
        if options.interactive && runNumber > 0 { options.output = originalOutput + "-" + UUID().uuidString }
        runNumber += 1
        if FileManager.default.fileExists(atPath: options.output + "/manifest.json") || FileManager.default.fileExists(atPath: options.output + "/host-status.json") { finishUI("failed", "Output already contains results; choose a fresh directory"); return }
        lifecycleEvents = []; measuredElapsed = 0
        lifecycleStart = visibleState()
        let startState = lifecycleStart!
        guard canMeasure(startState) else {
            finishUI("invalidated", "Start requires a visible, unoccluded window in the console session on an awake display")
            return
        }
        activity = ProcessInfo.processInfo.beginActivity(options: activityOptions, reason: "Renderer comparison measurement and result drain")
        options.backend = backend.titleOfSelectedItem!; options.fixture = fixture.titleOfSelectedItem!; options.mode = mode.titleOfSelectedItem!
        let dims = resolution.titleOfSelectedItem!.components(separatedBy: " × "); options.width = Int(dims[0])!; options.height = Int(dims[1])!
        let state = StopState(); active = state
        for c in [backend, fixture, resolution, mode] { c.isEnabled = false }; run.isEnabled = false; stop.isEnabled = true
        canvas.physicalSize = CGSize(width: options.width, height: options.height); canvas.layoutSubtreeIfNeeded(); canvas.metal.drawableSize = canvas.physicalSize; canvas.metal.contentsScale = window.backingScaleFactor; canvas.metal.displaySyncEnabled = options.mode == "paced"
        let screenID = (window.screen?.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber)?.uint32Value ?? CGMainDisplayID()
        var link: CVDisplayLink?; CVDisplayLinkCreateWithCGDisplay(screenID, &link)
        let period = link.map { CVDisplayLinkGetNominalOutputVideoRefreshPeriod($0) }
        let hz = period.flatMap { $0.timeValue > 0 && $0.flags == 0 ? Double($0.timeScale) / Double($0.timeValue) : nil }
        var config: [String: Any] = ["backend": options.backend, "fixture": options.fixture, "width": options.width, "height": options.height, "mode": options.mode, "warmup": options.warmup, "repeat_id": options.repeatID, "output": options.output, "display": ["refresh_hz": hz as Any? ?? NSNull(), "refresh_source": "CVDisplayLink nominal period; not observed presentation rate", "backing_scale": window.backingScaleFactor, "window_points_width": window.frame.width, "window_points_height": window.frame.height], "runtime": ["os": ProcessInfo.processInfo.operatingSystemVersionString, "cpu_count": ProcessInfo.processInfo.processorCount, "physical_memory_bytes": ProcessInfo.processInfo.physicalMemory], "power": ["thermal_before": ProcessInfo.processInfo.thermalState.rawValue, "low_power_mode": ProcessInfo.processInfo.isLowPowerModeEnabled]]
        var runtime = config["runtime"] as! [String: Any]
        runtime["lifecycle_start"] = startState
        runtime["window_level_policy"] = options.interactive ? "normal interactive window" : "temporary floating automatic benchmark window; restored after output"
        runtime["automatic_launch_settle_seconds"] = options.interactive ? 0.0 : 2.0
        runtime["activity_options"] = ["userInitiated", "latencyCritical", "idleDisplaySleepDisabled"]
        runtime["activity_options_raw"] = activityOptions.rawValue
        runtime["lifecycle_policy"] = "Invalidate full window occlusion, session resignation, system/display sleep; focus loss alone is allowed. Host status is authoritative through output completion."
        config["runtime"] = runtime
        if let d = options.duration { config["duration_seconds"] = d } else { config["frames_requested"] = options.frames }
        if let url = Bundle.main.url(forResource: "build", withExtension: "json"), let data = try? Data(contentsOf: url), let metadata = try? JSONSerialization.jsonObject(with: data) { config["build"] = metadata }
        guard let bytes = try? JSONSerialization.data(withJSONObject: config, options: [.sortedKeys]), let json = String(data: bytes, encoding: .utf8) else { finishUI("failed", "Cannot encode configuration"); return }
        setStatus("Initializing \(options.backend) · \(options.width) × \(options.height) physical pixels · \(options.mode)\n\(options.output)")
        let captured = options, layer = canvas.metal
        worker.async {
            var handle: UnsafeMutableRawPointer? = nil
            var result = "complete", detail = "", elapsed = 0.0
            func errorText() -> String { guard let p = rc_error(handle) else { return "Unknown renderer error" }; return String(cString: p) }
            func phase(_ text: String) { DispatchQueue.main.async { if state.read() == nil { self.setStatus(text) } } }
            handle = json.withCString { rc_create($0, Unmanaged.passUnretained(layer).toOpaque()) }
            if handle == nil { result = "failed"; detail = errorText() }
            else {
                func step(_ p: UInt32, _ f: UInt64) -> Bool { autoreleasepool { if let stop = state.read() { result = stop.0; detail = stop.1; return false }; if rc_step(handle, p, f) != 0 { result = "failed"; detail = errorText(); return false }; return true } }
                if step(0, 0) {
                    phase("Warming \(captured.backend) · \(captured.warmup) frames")
                    for f in 0..<captured.warmup { if !step(1, UInt64(f)) { break } }
                }
                if result == "complete" {
                    phase("Measuring \(captured.backend) · \(captured.fixture) · \(captured.mode) · \(captured.width) × \(captured.height)")
                    let begin = ProcessInfo.processInfo.systemUptime; var f: UInt64 = 0
                    while captured.duration.map({ ProcessInfo.processInfo.systemUptime - begin < $0 }) ?? (f < UInt64(captured.frames)) {
                        if !step(2, f) { break }; f += 1
                    }
                    elapsed = ProcessInfo.processInfo.systemUptime - begin
                }
                if let stop = state.read(), result == "complete" { result = stop.0; detail = stop.1 }
                if result.withCString({ rc_finish(handle, $0, elapsed) }) != 0 { result = "failed"; detail += " " + errorText() }
                rc_destroy(handle)
            }
            let finalResult = result, finalDetail = detail, finalElapsed = elapsed
            DispatchQueue.main.async {
                self.measuredElapsed = finalElapsed
                // An interruption during rc_finish/readback still invalidates the trial.
                // The retained renderer manifest can already say complete; host-status
                // is the authoritative host lifecycle verdict for the whole operation.
                let interruption = state.read()
                self.finishUI(finalResult == "complete" ? interruption?.0 ?? finalResult : finalResult,
                              finalResult == "complete" ? interruption?.1 ?? finalDetail : finalDetail)
            }
        }
    }
    func finishUI(_ suppliedResult: String, _ suppliedDetail: String) {
        precondition(Thread.isMainThread)
        var result = suppliedResult, detail = suppliedDetail
        let endState = visibleState()
        if result == "complete", !canMeasure(endState) {
            result = "invalidated"; detail = "Window/session/display not eligible at output completion"
            lifecycleEvents.append(["event": detail, "invalidating": true, "state": endState])
        }
        if let startState = lifecycleStart {
            let extra: [String: Any] = ["status": result, "reason": detail,
                "thermal_after": ProcessInfo.processInfo.thermalState.rawValue,
                "measured_elapsed_seconds": measuredElapsed, "lifecycle_start": startState,
                "lifecycle_end": endState, "lifecycle_events": lifecycleEvents,
                "activity_options": ["userInitiated", "latencyCritical", "idleDisplaySleepDisabled"],
                "activity_held_through_output": activity != nil,
                "window_level_policy": options.interactive ? "normal interactive window" : "temporary floating automatic benchmark window; restored after output"]
            do {
                let data = try JSONSerialization.data(withJSONObject: extra, options: [.prettyPrinted, .sortedKeys])
                try FileManager.default.createDirectory(atPath: options.output, withIntermediateDirectories: true)
                try data.write(to: URL(fileURLWithPath: options.output).appendingPathComponent("host-status.json"), options: .atomic)
            } catch {
                lifecycleStart = nil
                finishUI("failed", "Cannot write host lifecycle evidence: \(error)")
                return
            }
        }
        lifecycleStart = nil
        if let activity { ProcessInfo.processInfo.endActivity(activity); self.activity = nil }

        window.level = .normal
        active = nil; for c in [backend, fixture, resolution, mode] { c.isEnabled = true }; run.isEnabled = true; stop.isEnabled = false
        setStatus("\(result.capitalized)\(detail.isEmpty ? "" : ": " + detail)\n\(options.output)")
        exitCode = result == "complete" ? 0 : 1
        print("\(result): \(options.output) \(detail)"); fflush(stdout)
        if !options.interactive || closing { NSApp.terminate(nil) }
    }
}
do {
    let options = try Options.parse(); let app = NSApplication.shared; app.setActivationPolicy(.regular)
    let host = Host(options); app.delegate = host; app.run(); exit(host.exitCode)
} catch { fputs("RendererComparison: \(error)\n", stderr); exit(2) }
