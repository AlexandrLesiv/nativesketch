import AppKit
import Metal
import QuartzCore
import CryptoKit

// Isolated SOFTWARE replay experiment. No callback below measures physical photons.
func monotonicNow() -> Double { CACurrentMediaTime() }
let hostLaunchTime = monotonicNow()
enum Failure: Error { case message(String) }
func require(_ condition: Bool, _ message: String) throws { if !condition { throw Failure.message(message) } }
func tupleValues<T>(_ value: T) -> [Double] { withUnsafeBytes(of: value) { Array($0.bindMemory(to: Double.self)) } }
func screenID(_ screen: NSScreen) -> CGDirectDisplayID { (screen.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber)?.uint32Value ?? 0 }
func screens() -> [[String: Any]] { NSScreen.screens.map { s in
    let id = screenID(s), mode = CGDisplayCopyDisplayMode(id)
    return ["id": id, "name": s.localizedName, "scale": s.backingScaleFactor,
            "frame_points": NSStringFromRect(s.frame), "visible_frame_points": NSStringFromRect(s.visibleFrame),
            "pixel_width": mode?.pixelWidth ?? 0, "pixel_height": mode?.pixelHeight ?? 0,
            "nominal_refresh_hz": mode?.refreshRate ?? 0, "mirrors_display": CGDisplayMirrorsDisplay(id),
            "in_mirror_set": CGDisplayIsInMirrorSet(id) != 0]
} }
func jsonData(_ value: Any) throws -> Data { try JSONSerialization.data(withJSONObject: value, options: [.sortedKeys, .prettyPrinted]) }
struct Options {
    var backend = "", policy = "", scene = "dense1000", trace = "short", output = ""
    var width = 1600, height = 1000, display: UInt32 = 0
    var duration = 30.0, warmup = 5.0, idle = 0.0
    var startState = "warm"
    var holdResults = false
    var interactive = false, selfTest = false, logging = true
    static func parse() throws -> Options {
        var o = Options(), i = 1; let a = CommandLine.arguments; var seen = Set<String>()
        while i < a.count {
            let key = a[i]; i += 1
            if key == "--hold-results" { o.holdResults = true; continue }
            if key == "--interactive" { o.interactive = true; continue }
            if key == "--self-test" { o.selfTest = true; continue }
            if key == "--logging-disabled" { o.logging = false; continue }
            try require(i < a.count && seen.insert(key).inserted, "Missing/duplicate option \(key)")
            let v = a[i]; i += 1
            switch key {
            case "--backend": o.backend = v
            case "--policy": o.policy = v
            case "--scene": o.scene = v
            case "--trace": o.trace = v
            case "--start-state": o.startState = v
            case "--instrumentation": try require(["minimal", "full"].contains(v), "Invalid instrumentation"); o.logging = v == "full"
            case "--output": o.output = v
            case "--width": o.width = Int(v) ?? 0
            case "--height": o.height = Int(v) ?? 0
            case "--screen-id": o.display = UInt32(v) ?? 0
            case "--duration-seconds": o.duration = Double(v) ?? 0
            case "--warmup-seconds": o.warmup = Double(v) ?? -1
            case "--idle-seconds": o.idle = Double(v) ?? -1
            default: throw Failure.message("Unknown option \(key)")
            }
        }
        try require(["skia", "coregraphics"].contains(o.backend) && ["R", "C"].contains(o.policy), "Specify --backend skia|coregraphics --policy R|C")
        try require(o.display != 0 && (64...8192).contains(o.width) && (64...8192).contains(o.height), "Specify --screen-id and valid dimensions")
        try require(o.duration.isFinite && o.duration > 0 && o.duration <= 3600 && o.warmup.isFinite && o.warmup >= 0 && o.idle.isFinite && o.idle >= 0, "Invalid durations")
        try require(["warm", "first-use", "after-idle"].contains(o.startState), "Invalid start-state")
        if o.startState == "first-use" { o.warmup = 0; o.idle = 0 }
        if o.startState == "after-idle" && o.idle == 0 { o.idle = 30 }
        try require(o.output.hasPrefix("/"), "--output must be absolute")
        return o
    }
}
final class WakeGate {
    private let lock = NSLock(); private var pending = false
    func take() -> Bool { lock.lock(); defer { lock.unlock() }; if pending { return false }; pending = true; return true }
    func release() { lock.lock(); pending = false; lock.unlock() }
}
final class Canvas: NSView {
    var painter: ((CGContext?) -> Void)?
    var input: ((NSEvent, String) -> Void)?
    var isMetal = false
    override var isFlipped: Bool { true }
    // CAMetalLayer owns presentation; AppKit does not promise draw/updateLayer
    // callbacks for this backing-layer class. Metal invalidation is dispatched
    // explicitly on the same main runloop, while CG retains NSView.draw semantics.
    func requestPaint() {
        // Defer both requests beyond any current draw callback. AppKit can clear
        // needsDisplay set from inside draw(_:) during the warm/reset handoff.
        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            if self.isMetal { self.painter?(nil) } else { self.needsDisplay = true }
        }
    }
    override var isOpaque: Bool { true }
    override var acceptsFirstResponder: Bool { true }
    override func makeBackingLayer() -> CALayer { isMetal ? CAMetalLayer() : CALayer() }
    override func draw(_ dirtyRect: NSRect) { painter?(isMetal ? nil : NSGraphicsContext.current?.cgContext) }
    override func mouseDown(with e: NSEvent) { input?(e, "down") }
    override func mouseDragged(with e: NSEvent) { input?(e, "move") }
    override func mouseUp(with e: NSEvent) { input?(e, "up") }
    override func tabletPoint(with e: NSEvent) { input?(e, "tablet") }
    override func cancelOperation(_ sender: Any?) { NSApp.sendAction(#selector(Host.stopRun), to: nil, from: self) }
}
final class Host: NSObject, NSApplicationDelegate, NSWindowDelegate {
    let o: Options, canvas = Canvas(), status = NSTextField(wrappingLabelWithString: "Software replay — physical latency NOT measured")
    let run = NSButton(title: "Run", target: nil, action: nil), stop = NSButton(title: "Stop", target: nil, action: nil)
    let results = NSButton(title: "Show Results", target: nil, action: nil)
    var closeAfterFinished = false
    let colorSpace = CGColorSpace(name: CGColorSpace.sRGB)!, gate = WakeGate()
    var window: NSWindow!, target: NSScreen!, device: MTLDevice?, queue: MTLCommandQueue?
    var replay: UnsafeMutableRawPointer?, skia: UnsafeMutableRawPointer?
    var watchdog: DispatchSourceTimer?
    var timer: DispatchSourceTimer?, activity: NSObjectProtocol?, observations: [NSObjectProtocol] = []
    var active = false, finalizing = false, measured = false, outcome = "complete", reason = ""
    var epoch = 0.0, initialFrame = NSRect.zero, initialMode: [Int] = [], rows: [[String: Any]] = [], events: [[String: Any]] = []
    var advanceCount = 0, advanceCPUms = 0.0, inputTotalMs = 0.0, outlineTotalMs = 0.0, exportTotalMs = 0.0
    var measuredFrames = 0, drawCPUTotalMs = 0.0, drawCPUMaxMs = 0.0
    var measuredCPUStart: (user: Double, system: Double)?
    var deliveries: [[String: Any]] = [], lastDrawSample: UInt64 = 0, lastDrawTime = 0.0, latestReceipt = 0.0
    var inputEvents: [[String: Any]] = [], presentations: [[String: Any]] = []
    var frameIndex: UInt64 = 0, requestTime = 0.0, requestPending = false, generation: UInt64 = 0
    var basePaths: [(CGPath, CGColor)] = [], livePaths: [(CGPath, CGColor)] = []
    var baseRevision = UInt64.max, liveRevision = UInt64.max, cacheKey = "", cacheImage: CGImage?
    var finalImage: CGImage?, lastTexture: MTLTexture?, terminalDrawn = false
    var initialState: [String: Any] = [:], finalMetadata: [String: Any] = [:]
    init(_ options: Options) { o = options }
    func write(_ value: Any, _ name: String) throws { try jsonData(value).write(to: URL(fileURLWithPath: o.output).appendingPathComponent(name), options: .atomic) }
    func mode() -> [Int] { guard let m = CGDisplayCopyDisplayMode(o.display) else { return [] }; return [m.pixelWidth, m.pixelHeight, Int(m.refreshRate * 1000)] }
    func check() throws {
        try require(screenID(window.screen ?? target) == o.display, "Canvas left selected display")
        try require(CGDisplayIsInMirrorSet(o.display) == 0, "Display mirroring invalidates comparison")
        let b = canvas.convertToBacking(canvas.bounds)
        try require(abs(b.width - Double(o.width)) < 0.01 && abs(b.height - Double(o.height)) < 0.01, "Canvas backing dimensions changed")
        try require(target.visibleFrame.insetBy(dx: -0.5, dy: -0.5).contains(window.frame), "Full canvas/window does not fit target display")
        try require(window.isVisible && !window.isMiniaturized && window.occlusionState.contains(.visible), "Window not visible")
        if active || finalizing {
            if monotonicNow() - lastOcclusionCheck >= 0.25 {
                lastOcclusionCheck = monotonicNow(); occlusionChecks += 1
                if let info = CGWindowListCopyWindowInfo([.optionOnScreenAboveWindow, .excludeDesktopElements], CGWindowID(window.windowNumber)) as? [[String: Any]], let primary = NSScreen.screens.first {
                    let canvasOnScreen = window.convertToScreen(canvas.convert(canvas.bounds, to: nil))
                    let own = CGRect(x: canvasOnScreen.minX, y: primary.frame.maxY-canvasOnScreen.maxY, width: canvasOnScreen.width, height: canvasOnScreen.height)
                    for entry in info {
                        guard (entry[kCGWindowOwnerPID as String] as? NSNumber)?.int32Value != ProcessInfo.processInfo.processIdentifier,
                              (entry[kCGWindowAlpha as String] as? NSNumber)?.doubleValue ?? 0 > 0.01,
                              let dictionary = entry[kCGWindowBounds as String] as? [String: Any],
                              let rect = CGRect(dictionaryRepresentation: dictionary as CFDictionary), rect.intersects(own) else { continue }
                        throw Failure.message("Another onscreen window overlaps measured canvas (window \(entry[kCGWindowNumber as String] ?? "unknown"))")
                    }
                } else { occlusionUnavailable += 1 }
            }
            try require(window.frame == initialFrame && mode() == initialMode, "Window geometry/display mode changed")
        }
    }
    func applicationDidFinishLaunching(_ note: Notification) {
        do {
            try require(!FileManager.default.fileExists(atPath: o.output), "Output already exists; preserve earlier evidence")
            try FileManager.default.createDirectory(atPath: o.output, withIntermediateDirectories: true)
            guard let s = NSScreen.screens.first(where: { screenID($0) == o.display }) else { throw Failure.message("Display not found") }; target = s
            try require(CGDisplayIsInMirrorSet(o.display) == 0, "Disable mirroring before measurement")
            let w = Double(o.width) / s.backingScaleFactor, h = Double(o.height) / s.backingScaleFactor
            window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: max(w + 24, 620), height: h + 90), styleMask: [.titled, .closable, .miniaturizable], backing: .buffered, defer: false)
            window.delegate = self; window.title = "Software replay — \(o.backend) / \(o.policy) — physical latency pending"
            window.colorSpace = NSColorSpace.sRGB
            let content = NSView(); window.contentView = content
            canvas.isMetal = o.backend == "skia"; canvas.frame = NSRect(x: 12, y: 12, width: w, height: h); canvas.wantsLayer = true
            canvas.layerContentsRedrawPolicy = .onSetNeedsDisplay
            canvas.setAccessibilityElement(true); canvas.setAccessibilityLabel("Software replay canvas; physical pen latency not measured")
            if let layer = canvas.layer as? CAMetalLayer {
                guard let d = MTLCreateSystemDefaultDevice(), let q = d.makeCommandQueue() else { throw Failure.message("Metal unavailable") }
                device = d; queue = q; layer.device = d; layer.pixelFormat = .bgra8Unorm; layer.colorspace = colorSpace
                layer.framebufferOnly = false; layer.drawableSize = CGSize(width: o.width, height: o.height)
                layer.contentsScale = s.backingScaleFactor; layer.maximumDrawableCount = 3; layer.displaySyncEnabled = true; layer.isOpaque = true
            }
            status.frame = NSRect(x: 12, y: h + 20, width: max(w - 285, 290), height: 50)
            results.frame = NSRect(x: max(w - 265, 320), y: h + 28, width: 100, height: 32)
            results.target = self; results.action = #selector(showResults); results.isEnabled = false
            run.frame = NSRect(x: max(w - 160, 430), y: h + 28, width: 75, height: 32)
            stop.frame = NSRect(x: max(w - 80, 510), y: h + 28, width: 75, height: 32)
            run.target = self; run.action = #selector(start); stop.target = self; stop.action = #selector(stopRun); stop.isEnabled = false
            stop.keyEquivalent = "."; stop.keyEquivalentModifierMask = .command
            status.stringValue = "Ready — \(o.backend) / \(o.policy) · \(o.scene) / \(o.trace)\nSynthetic replay · \(s.localizedName) · \(o.width) × \(o.height) pixels\nPhysical pen-to-screen latency NOT measured"
            status.setAccessibilityLabel("Experiment configuration and status")
            canvas.setAccessibilityLabel("\(o.backend) policy \(o.policy), \(o.scene) \(o.trace) synthetic replay, \(o.width) by \(o.height) physical pixels on \(s.localizedName); physical latency not measured")
            for v in [canvas, status, results, run, stop] { content.addSubview(v) }
            let menu = NSMenu(), item = NSMenuItem(), appMenu = NSMenu(); appMenu.addItem(withTitle: "Quit", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q"); item.submenu = appMenu; menu.addItem(item); NSApp.mainMenu = menu
            canvas.painter = { [weak self] c in self?.draw(c) }
            canvas.input = { [weak self] e, phase in self?.nativeInput(e, phase) }
            window.setFrameOrigin(NSPoint(x: s.visibleFrame.midX - window.frame.width / 2, y: s.visibleFrame.midY - window.frame.height / 2))
            window.makeKeyAndOrderFront(nil); NSApp.activate(ignoringOtherApps: true); window.makeFirstResponder(canvas)
            for n in [NSWorkspace.sessionDidResignActiveNotification, NSWorkspace.willSleepNotification, NSWorkspace.screensDidSleepNotification] {
                observations.append(NSWorkspace.shared.notificationCenter.addObserver(forName: n, object: nil, queue: .main) { [weak self] _ in self?.invalidate(n.rawValue) })
            }
            observations.append(NotificationCenter.default.addObserver(forName: NSApplication.didChangeScreenParametersNotification, object: nil, queue: .main) { [weak self] _ in self?.invalidate("Display configuration changed") })
            if !o.interactive { DispatchQueue.main.asyncAfter(deadline: .now() + 1) { self.start() } }
        } catch { abort(error) }
    }
    func nativeInput(_ e: NSEvent, _ phase: String) {
        let p = canvas.convert(e.locationInWindow, from: nil)
        inputEvents.append(["native_timestamp_s": e.timestamp, "receipt_s": monotonicNow(), "phase": phase, "x_points": p.x, "y_points": p.y, "pressure": e.pressure, "type": e.type.rawValue, "subtype": e.subtype.rawValue])
        if active { invalidate("Physical input during deterministic replay") }
    }
    func invalidate(_ why: String) { guard active || finalizing else { return }; events.append(["time_s": monotonicNow(), "reason": why]); outcome = "invalid"; reason = why; if active { finish() } }
    func windowDidMove(_ n: Notification) { invalidate("Window moved") }
    func windowDidResize(_ n: Notification) { invalidate("Window resized") }
    func windowDidMiniaturize(_ n: Notification) { invalidate("Window minimized") }
    func windowDidChangeScreen(_ n: Notification) { invalidate("Window screen changed") }
    func windowDidChangeBackingProperties(_ n: Notification) { invalidate("Backing scale changed") }
    func windowDidChangeOcclusionState(_ n: Notification) { if !window.occlusionState.contains(.visible) { invalidate("Window occluded") } }
    func windowShouldClose(_ sender: NSWindow) -> Bool {
        if active || finalizing { closeAfterFinished = true; stopRun(); return false }
        DispatchQueue.main.async { NSApp.terminate(nil) }; return true
    }
    @objc func showResults() { NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: o.output, isDirectory: true)]) }
    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply { if active || finalizing { stopRun(); return .terminateCancel }; return .terminateNow }
    @objc func stopRun() { guard active || finalizing else { return }; outcome = "cancelled"; reason = "Stop requested"; if active { finish() } }
    func abort(_ error: Error) { fputs("\(error)\n", stderr); if active || finalizing { outcome = "failed"; reason = "\(error)"; if active { finish() } } else { exit(1) } }
    var initialization: [[String: Any]] = [], nativeCTM: [Double] = [], retainedClip: CGPath?
    var lastStatusUpdate = 0.0, lastOcclusionCheck = 0.0
    var occlusionChecks = 0, occlusionUnavailable = 0
    var requestedViewport: UInt64 = .max
    var preparingMeasured = false
    var snapshot = PcSnapshot()
    func ffiCheck(_ result: Int32) throws { try require(result == 0, String(cString: pc_error())) }
    func hashString<T>(_ tuple: T) -> String { withUnsafeBytes(of: tuple) { String(cString: $0.baseAddress!.assumingMemoryBound(to: CChar.self)) } }
    func resetReplay(_ duration: Double) throws {
        let constructionStart = monotonicNow()
        if let replay { pc_replay_destroy(replay) }
        let config: [String: Any] = ["scene": o.scene, "trace": o.trace, "width": o.width, "height": o.height, "duration_s": duration, "sample_hz": 240]
        let data = try jsonData(config), text = String(decoding: data, as: UTF8.self)
        replay = text.withCString { pc_replay_create($0) }; try require(replay != nil, String(cString: pc_error()))
        let constructed = monotonicNow(); snapshot = PcSnapshot()
        try ffiCheck(pc_replay_initial_snapshot(replay, &snapshot))
        try require(snapshot.total_samples == 0 && snapshot.live_outline_count == 0, "Initial cache snapshot must not consume replay input")
        initialization.append(["stage": "replay_creation_and_reference_oracle", "duration_ms": (constructed-constructionStart)*1000, "initial_snapshot_ms": (monotonicNow()-constructed)*1000, "initial_sample_applied": false])
        requestedViewport = .max; baseRevision = .max; liveRevision = .max; cacheKey = ""; cacheImage = nil; decodedImage = nil; retainedClip = nil
        if let skia { try ffiCheck(pc_skia_reset(skia)) }
        if measured { beginMeasuredEpoch() } else { epoch = monotonicNow() }; latestReceipt = epoch; terminalDrawn = false
    }
    @objc func start() {
        guard !active && !finalizing else { return }
        do {
            try check(); initialFrame = window.frame; initialMode = mode()
            initialState = ["thermal_state": ProcessInfo.processInfo.thermalState.rawValue, "low_power_mode": ProcessInfo.processInfo.isLowPowerModeEnabled, "selected_screen_id": o.display, "actual_screen_id": screenID(window.screen ?? target), "screens": screens(), "window_frame": NSStringFromRect(window.frame), "canvas_points": NSStringFromRect(canvas.bounds), "scale": target.backingScaleFactor, "physical_width": o.width, "physical_height": o.height]
            if let d = device, let q = queue {
                let begin = monotonicNow()
                skia = pc_skia_create(Unmanaged.passUnretained(d as AnyObject).toOpaque(), Unmanaged.passUnretained(q as AnyObject).toOpaque(), UInt32(o.width), UInt32(o.height))
                try require(skia != nil, String(cString: pc_error()))
                initialization.append(["stage": "skia_context_creation", "duration_ms": (monotonicNow()-begin)*1000])
            }
            measured = o.warmup == 0; try resetReplay(measured ? o.duration : o.warmup)
            active = true; run.isEnabled = false; stop.isEnabled = true
            activity = ProcessInfo.processInfo.beginActivity(options: [.userInitiated, .latencyCritical, .idleDisplaySleepDisabled], reason: "Authorized visible Cintiq software replay")
            startTimer()
            let watcher = DispatchSource.makeTimerSource(queue: .main)
            watcher.schedule(deadline: .now() + .milliseconds(250), repeating: .milliseconds(250))
            watcher.setEventHandler { [weak self] in
                guard let self, self.active else { return }
                if self.requestPending && monotonicNow() - self.requestTime > 2 {
                    self.abort(Failure.message("Redraw watchdog: pending native paint exceeded two seconds"))
                } else if self.timer != nil && monotonicNow() - self.epoch > (self.measured ? self.o.duration : self.o.warmup) + 10 {
                    self.abort(Failure.message("Replay watchdog: no terminal paint within ten seconds of trace duration"))
                }
            }
            watchdog = watcher; watcher.resume()
            if o.selfTest { DispatchQueue.main.asyncAfter(deadline: .now() + 0.4) { self.stop.performClick(nil) } }
        } catch { abort(error) }
    }
    func startTimer() {
        let t = DispatchSource.makeTimerSource(queue: DispatchQueue(label: "replay-deadlines", qos: .userInteractive))
        t.schedule(deadline: .now(), repeating: .milliseconds(1), leeway: .microseconds(100))
        t.setEventHandler { [weak self] in
            guard let self, self.gate.take() else { return }
            DispatchQueue.main.async { self.gate.release(); self.advance() }
        }; timer = t; t.resume()
    }
    func advance() {
        guard active && timer != nil else { return }
        do {
            try check()
            let receipt = monotonicNow()
            let elapsed = receipt - epoch
            try ffiCheck(pc_replay_advance(replay, elapsed, &snapshot))
            if measured {
                advanceCount += 1; advanceCPUms += (monotonicNow()-receipt)*1000
                inputTotalMs += snapshot.input_ms; outlineTotalMs += snapshot.outline_ms; exportTotalMs += snapshot.export_ms
            }
            if snapshot.delivered_now > 0 {
                latestReceipt = receipt
                if measured && o.logging { deliveries.append(["receipt_s": receipt, "epoch_s": epoch, "first_sample_id": snapshot.last_sample_id - snapshot.delivered_now + 1, "last_sample_id": snapshot.last_sample_id, "first_scheduled_s": snapshot.first_delivered_time_s, "last_scheduled_s": snapshot.last_delivered_time_s, "sample_count": snapshot.delivered_now, "input_ms": snapshot.input_ms, "outline_ms": snapshot.outline_ms, "export_ms": snapshot.export_ms]) }
            }
            if snapshot.delivered_now > 0 || snapshot.viewport_revision != requestedViewport || snapshot.trace_complete != 0 || frameIndex == 0 {
                requestedViewport = snapshot.viewport_revision
                if !requestPending { requestTime = monotonicNow(); requestPending = true; canvas.requestPaint() }
            }
        } catch { abort(error) }
    }
    func path(_ commands: UnsafePointer<RcCommand>, _ start: Int, _ count: Int) -> CGPath {
        let result = CGMutablePath()
        for i in start..<(start + count) {
            let c = commands[i], p = tupleValues(c.points)
            switch c.verb {
            case 0: result.move(to: CGPoint(x: p[0], y: p[1]))
            case 1: result.addLine(to: CGPoint(x: p[0], y: p[1]))
            case 2: result.addCurve(to: CGPoint(x: p[4], y: p[5]), control1: CGPoint(x: p[0], y: p[1]), control2: CGPoint(x: p[2], y: p[3]))
            default: result.closeSubpath()
            }
        }; return result
    }
    func paths(_ commands: UnsafePointer<RcCommand>?, _ outlines: UnsafePointer<RcOutline>?, _ count: UInt64) -> [(CGPath, CGColor)] {
        guard let commands, let outlines else { return [] }
        return (0..<Int(count)).map { i in let o = outlines[i]; return (path(commands, Int(o.command_start), Int(o.command_count)), CGColor(colorSpace: colorSpace, components: tupleValues(o.rgba).map { CGFloat($0) })!) }
    }
    func transform(_ c: CGContext) {
        let t = tupleValues(snapshot.transform); c.concatenate(CGAffineTransform(a: t[0], b: t[1], c: t[2], d: t[3], tx: t[4], ty: t[5]))
        if snapshot.has_clip != 0, let retainedClip { c.clip(to: CGRect(x: 150, y: 130, width: 1300, height: 740)); c.addPath(retainedClip); c.clip(using: .winding) }
    }
    func paintPaths(_ c: CGContext, _ paths: [(CGPath, CGColor)]) { for (p, color) in paths { c.addPath(p); c.setFillColor(color); c.fillPath(using: .winding) } }
    var decodedImage: CGImage?
    func paintBase(_ c: CGContext) {
        c.saveGState(); transform(c)
        if snapshot.has_image != 0 {
            if decodedImage == nil, let bytes = snapshot.image_rgba {
                let data = Data(bytes: bytes, count: Int(snapshot.image_byte_count))
                if let provider = CGDataProvider(data: data as CFData) { decodedImage = CGImage(width: 2048, height: 2048, bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: 8192, space: colorSpace, bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.last.rawValue), provider: provider, decode: nil, shouldInterpolate: true, intent: .defaultIntent) }
            }
            if let image = decodedImage {
                c.saveGState(); let t = tupleValues(snapshot.image_transform); c.concatenate(CGAffineTransform(a: t[0], b: t[1], c: t[2], d: t[3], tx: t[4], ty: t[5])); c.translateBy(x: 0, y: 2048); c.scaleBy(x: 1, y: -1); c.draw(image, in: CGRect(x: 0, y: 0, width: 2048, height: 2048)); c.restoreGState()
            }
        }
        paintPaths(c, basePaths); c.restoreGState()
    }
    func bitmap() throws -> CGContext {
        guard let c = CGContext(data: nil, width: o.width, height: o.height, bitsPerComponent: 8, bytesPerRow: o.width * 4, space: colorSpace, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else { throw Failure.message("Bitmap allocation failed") }
        c.translateBy(x: 0, y: CGFloat(o.height)); c.scaleBy(x: 1, y: -1); return c
    }
    func background(_ c: CGContext) { c.setFillColor(CGColor(gray: 1, alpha: 1)); c.fill(CGRect(x: 0, y: 0, width: o.width, height: o.height)); c.setAllowsAntialiasing(true); c.setShouldAntialias(true); c.interpolationQuality = .low }
    func cgPaint(_ c: CGContext) throws -> PcPaintMetrics {
        var m = PcPaintMetrics(); var begin = monotonicNow()
        if retainedClip == nil && snapshot.has_clip != 0, let commands = snapshot.clip_commands { retainedClip = path(commands, 0, Int(snapshot.clip_command_count)) }
        if baseRevision != snapshot.document_revision { basePaths = paths(snapshot.base_commands, snapshot.base_outlines, snapshot.base_outline_count); baseRevision = snapshot.document_revision }
        if liveRevision != snapshot.live_revision { livePaths = paths(snapshot.live_commands, snapshot.live_outlines, snapshot.live_outline_count); liveRevision = snapshot.live_revision }
        m.path_ms = (monotonicNow() - begin) * 1000; begin = monotonicNow()
        let cached = o.policy == "C" && snapshot.cache_allowed != 0
        let key = "\(snapshot.document_revision):\(snapshot.viewport_revision):\(o.width):\(o.height):sRGB"
        if cached && key != cacheKey {
            let b = try bitmap(); background(b); paintBase(b)
            guard let image = b.makeImage() else { throw Failure.message("Cache snapshot failed") }
            cacheImage = image; cacheKey = key; m.cache_rebuilt = 1
        }
        if !cached { cacheImage = nil; cacheKey = "" }
        m.cache_bytes = cacheImage == nil ? 0 : UInt64(o.width * o.height * 4)
        m.cache_build_ms = (monotonicNow() - begin) * 1000; begin = monotonicNow()
        background(c)
        if cached, let image = cacheImage { c.saveGState(); c.translateBy(x: 0, y: CGFloat(o.height)); c.scaleBy(x: 1, y: -1); c.draw(image, in: CGRect(x: 0, y: 0, width: o.width, height: o.height)); c.restoreGState() } else { paintBase(c) }
        c.saveGState(); transform(c); paintPaths(c, livePaths); c.restoreGState()
        m.paint_submit_ms = (monotonicNow() - begin) * 1000; return m
    }
    func draw(_ context: CGContext?) {
        guard active && requestPending else { return }
        requestPending = false
        do {
            try check(); let started = monotonicNow(); var metrics = PcPaintMetrics(), acquire = 0.0, presentEnqueue = 0.0
            let sample = snapshot.last_sample_id, sampleTime = snapshot.last_sample_time_s, frame = frameIndex
            if o.backend == "coregraphics" {
                guard let c = context else { throw Failure.message("No AppKit CGContext") }
                if nativeCTM.isEmpty { let t = c.ctm; nativeCTM = [t.a,t.b,t.c,t.d,t.tx,t.ty].map { Double($0) } }
                c.saveGState(); c.scaleBy(x: 1 / target.backingScaleFactor, y: 1 / target.backingScaleFactor); metrics = try cgPaint(c); c.restoreGState()
            } else {
                let a = monotonicNow(); guard let layer = canvas.layer as? CAMetalLayer, let drawable = layer.nextDrawable(), let q = queue else { throw Failure.message("No Metal drawable") }; acquire = (monotonicNow() - a) * 1000
                try ffiCheck(pc_skia_render(skia, Unmanaged.passUnretained(drawable.texture as AnyObject).toOpaque(), &snapshot, o.policy == "C" ? 1 : 0, &metrics))
                lastTexture = drawable.texture
                let p = monotonicNow(); guard let command = q.makeCommandBuffer() else { throw Failure.message("No presentation command") }
                let epochForFrame = epoch, phaseForFrame = measured
                drawable.addPresentedHandler { [weak self] d in DispatchQueue.main.async { self?.presentations.append(["frame": frame, "measured": phaseForFrame, "sample_id": sample, "presented_time_s": d.presentedTime, "sample_age_at_presented_ms": d.presentedTime > 0 ? (d.presentedTime - epochForFrame - sampleTime) * 1000 : NSNull()]) } }
                command.addCompletedHandler { [weak self] buffer in DispatchQueue.main.async { self?.presentations.append(["frame": frame, "measured": phaseForFrame, "completion_callback_s": monotonicNow(), "command_status": buffer.status.rawValue]) } }
                command.present(drawable); command.commit(); presentEnqueue = (monotonicNow() - p) * 1000
            }
            let ended = monotonicNow()
            if measured {
                measuredFrames += 1
                let duration = (ended-started)*1000
                drawCPUTotalMs += duration; drawCPUMaxMs = max(drawCPUMaxMs, duration)
            }
            if measured && o.logging {
                rows.append(["frame": frame, "draw_start_s": started, "host_launch_to_draw_ms": (started-hostLaunchTime)*1000, "draw_return_s": ended, "request_to_draw_ms": (started-requestTime)*1000,
                    "sample_id": sample, "previous_draw_sample_id": lastDrawSample, "newly_represented_samples": sample >= lastDrawSample ? sample-lastDrawSample : 0, "latest_receipt_to_draw_ms": (started-latestReceipt)*1000, "draw_interval_ms": lastDrawTime == 0 ? NSNull() : (started-lastDrawTime)*1000, "epoch_s": epoch, "sample_scheduled_s": sampleTime, "sample_age_at_draw_ms": (started-epoch-sampleTime)*1000,
                    "total_samples": snapshot.total_samples, "document_revision": snapshot.document_revision, "live_revision": snapshot.live_revision,
                    "input_ms": snapshot.input_ms, "outline_ms": snapshot.outline_ms, "export_ms": snapshot.export_ms,
                    "path_ms": metrics.path_ms, "cache_build_ms": metrics.cache_build_ms, "paint_submit_ms": metrics.paint_submit_ms,
                    "cache_rebuilt": metrics.cache_rebuilt, "cache_bytes": metrics.cache_bytes, "acquire_ms": acquire,
                    "present_enqueue_ms": presentEnqueue, "draw_cpu_ms": (ended-started)*1000, "geometry_hash": hashString(snapshot.geometry_hash)])
            }
            lastDrawSample = sample; lastDrawTime = started
            frameIndex += 1
            if ended - lastStatusUpdate >= 1 {
                lastStatusUpdate = ended
                status.stringValue = "\(measured ? "Measured" : "Warm-up") software replay — \(o.backend) / \(o.policy)\n\(snapshot.total_samples) samples; physical latency NOT measured"
            }
            if preparingMeasured {
                preparingMeasured = false; requestPending = false
                status.stringValue = "Cache ready; idle \(o.idle)s — physical latency pending"
                DispatchQueue.main.asyncAfter(deadline: .now() + o.idle) {
                    guard self.active else { return }; do { try self.check(); self.measured = true; self.frameIndex = 0; self.lastDrawSample = 0; self.lastDrawTime = 0; self.beginMeasuredEpoch(); self.startTimer() } catch { self.abort(error) }
                }
            } else if snapshot.trace_complete != 0 {
                if measured { terminalDrawn = true; finish() }
                else {
                    timer?.cancel(); timer = nil
                    try resetReplay(o.duration); preparingMeasured = true; requestTime = monotonicNow(); requestPending = true
                    canvas.requestPaint()
                }
            }
        } catch { abort(error) }
    }
    func capture() throws {
        if o.backend == "coregraphics" {
            let c = try bitmap(); _ = try cgPaint(c); finalImage = c.makeImage()
        } else if let texture = lastTexture, let q = queue, let d = device {
            let row = ((o.width * 4 + 255) / 256) * 256
            guard let buffer = d.makeBuffer(length: row * o.height, options: .storageModeShared), let command = q.makeCommandBuffer(), let blit = command.makeBlitCommandEncoder() else { throw Failure.message("Capture allocation failed") }
            blit.copy(from: texture, sourceSlice: 0, sourceLevel: 0, sourceOrigin: MTLOrigin(x: 0, y: 0, z: 0), sourceSize: MTLSize(width: o.width, height: o.height, depth: 1), to: buffer, destinationOffset: 0, destinationBytesPerRow: row, destinationBytesPerImage: row * o.height)
            blit.endEncoding(); command.commit(); command.waitUntilCompleted()
            try require(command.status == .completed, "Untimed readback failed")
            let data = Data(bytes: buffer.contents(), count: row * o.height)
            if let provider = CGDataProvider(data: data as CFData) {
                finalImage = CGImage(width: o.width, height: o.height, bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: row, space: colorSpace, bitmapInfo: [.byteOrder32Little, CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedFirst.rawValue)], provider: provider, decode: nil, shouldInterpolate: false, intent: .defaultIntent)
            }
        }
        guard let image = finalImage, let png = NSBitmapImageRep(cgImage: image).representation(using: .png, properties: [:]) else { throw Failure.message("No final image") }
        try png.write(to: URL(fileURLWithPath: o.output).appendingPathComponent("final.png"), options: .atomic)
        finalMetadata["capture_sha256"] = SHA256.hash(data: png).map { String(format: "%02x", $0) }.joined()
        finalMetadata["capture_kind"] = o.backend == "coregraphics" ? "Untimed bitmap proxy using identical painter; not presented pixels" : "Untimed final Metal texture readback; not optical pixels"
    }
    var gpuRetired = false, retirementFinished = false
    func retireGPU(_ continuation: @escaping () -> Void) {
        guard let queue else { gpuRetired = true; continuation(); return }
        guard let fence = queue.makeCommandBuffer() else { retirementFailure("Could not allocate GPU retirement fence"); return }
        let begin = monotonicNow()
        fence.label = "Untimed finalization retirement fence"
        fence.addCompletedHandler { [weak self] command in
            let status = command.status
            DispatchQueue.main.async {
                guard let self, !self.retirementFinished else { return }
                self.retirementFinished = true
                self.finalMetadata["gpu_retirement_ms"] = (monotonicNow()-begin)*1000
                self.finalMetadata["gpu_retirement_status"] = status.rawValue
                guard status == .completed else { self.retirementFailure("GPU retirement fence did not complete successfully"); return }
                self.gpuRetired = true; continuation()
            }
        }
        fence.commit()
        DispatchQueue.main.asyncAfter(deadline: .now() + 5) { [weak self] in
            guard let self, !self.retirementFinished else { return }
            self.retirementFinished = true; self.retirementFailure("GPU retirement fence timed out after five seconds")
        }
    }
    func retirementFailure(_ message: String) {
        // Do not destroy a context whose GPU work may still own its resources.
        // Preserve a failed result, then let process teardown retire the device.
        outcome = "failed"; reason = message
        finalMetadata["gpu_retirement"] = "unproven; context destruction skipped"
        try? write(rows, "frames.json"); try? write(deliveries, "deliveries.json")
        try? write(["schema_version": 1, "status": outcome, "reason": reason, "backend": o.backend, "policy": o.policy, "initial": initialState, "final": finalMetadata, "lifecycle": events], "manifest.json")
        fputs("\(message)\n", stderr); exit(1)
    }
    func processCPU() -> (user: Double, system: Double)? {
        var usage = rusage()
        guard getrusage(RUSAGE_SELF, &usage) == 0 else { return nil }
        return (Double(usage.ru_utime.tv_sec) + Double(usage.ru_utime.tv_usec) / 1_000_000,
                Double(usage.ru_stime.tv_sec) + Double(usage.ru_stime.tv_usec) / 1_000_000)
    }
    func beginMeasuredEpoch() {
        measuredCPUStart = processCPU()
        epoch = monotonicNow()
    }
    func finish() {
        guard active else { return }
        let cpuEnd = measured ? processCPU() : nil
        if let cpuStart = measuredCPUStart, let cpuEnd {
            finalMetadata["measured_cpu_user_s"] = cpuEnd.user - cpuStart.user
            finalMetadata["measured_cpu_system_s"] = cpuEnd.system - cpuStart.system
        } else {
            finalMetadata["measured_cpu_user_s"] = NSNull()
            finalMetadata["measured_cpu_system_s"] = NSNull()
        }
        finalMetadata["measured_cpu_scope"] = "Process user+system CPU across all process threads from measured replay epoch to finish entry, excluding initialization, warmup, idle, GPU retirement, capture and artifact serialization; not GPU or WindowServer CPU"
        finalMetadata["draw_cpu_total_ms"] = drawCPUTotalMs
        finalMetadata["draw_cpu_max_ms"] = drawCPUMaxMs
        finalMetadata["draw_cpu_count"] = measuredFrames
        finalMetadata["draw_cpu_scope"] = "Aggregate synchronous draw callback wall durations, including drawable acquisition and painter calls; collected in both instrumentation modes; not CPU utilization or photon latency"
        active = false; finalizing = true; timer?.cancel(); timer = nil; watchdog?.cancel(); watchdog = nil
        status.stringValue = "Finalizing software evidence — Stop remains available"
        finalMetadata["elapsed_s"] = monotonicNow() - epoch
        finalMetadata["measured_frames"] = measuredFrames
        finalMetadata["advance_totals"] = ["calls": advanceCount, "whole_call_ms": advanceCPUms, "input_ms": inputTotalMs, "outline_ms": outlineTotalMs, "export_ms": exportTotalMs]
        finalMetadata["frame_stage_scope"] = "Frame input/outline/export fields refer only to latest advance; use advance_totals for all calls including zero-delivery and commit-only work"
        // Keep Stop/lifecycle active while the same queue asynchronously retires on every exit path.
        retireGPU {
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) {
            do {
                try self.check()
                if self.terminalDrawn { try self.capture() }
                self.finalMetadata["geometry_hash"] = self.hashString(self.snapshot.geometry_hash)
                self.finalMetadata["document_hash"] = self.hashString(self.snapshot.document_hash)
                self.finalMetadata["total_samples"] = self.snapshot.total_samples
                self.finalMetadata["expected_samples"] = self.snapshot.expected_samples
                if let replay = self.replay, let p = pc_replay_metadata_json(replay), let data = String(cString: p).data(using: .utf8) {
                    let metadata = try JSONSerialization.jsonObject(with: data) as? [String: Any] ?? [:]
                    self.finalMetadata["replay"] = metadata
                    if self.outcome == "complete" {
                        try require(metadata["expected_final_geometry_hash"] as? String == self.hashString(self.snapshot.geometry_hash), "Final geometry differs from reference replay")
                        try require(metadata["expected_final_document_hash"] as? String == self.hashString(self.snapshot.document_hash), "Final document differs from reference replay")
                    }
                }
                if self.outcome == "complete" { try require(self.terminalDrawn && self.snapshot.total_samples == self.snapshot.expected_samples, "Terminal sample completeness failed") }
                try self.write(self.deliveries, "deliveries.json"); try self.write(self.rows, "frames.json"); try self.write(self.presentations, "metal-observations.json"); try self.write(self.inputEvents, "native-input.json")
                if let first = self.rows.first {
                    let keys = first.keys.sorted()
                    func field(_ v: Any) -> String { "\"" + String(describing: v).replacingOccurrences(of: "\"", with: "\"\"") + "\"" }
                    let csv = ([keys.joined(separator: ",")] + self.rows.map { r in keys.map { field(r[$0] ?? "") }.joined(separator: ",") }).joined(separator: "\n") + "\n"
                    try Data(csv.utf8).write(to: URL(fileURLWithPath: self.o.output).appendingPathComponent("frames.csv"), options: .atomic)
                }
            } catch { if self.outcome == "complete" { self.outcome = "failed"; self.reason = "\(error)" } }
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) { self.finalWrite() }
        }
    }
    }
    func finalWrite() {
        do {
            var usage = rusage()
            if getrusage(RUSAGE_SELF, &usage) == 0 {
                finalMetadata["process_lifetime_peak_rss_bytes"] = usage.ru_maxrss
                finalMetadata["memory_scope"] = "Darwin process-lifetime maximum resident set; includes initialization and untimed capture; excludes distinct GPU/WindowServer allocation accounting"
            } else { finalMetadata["process_lifetime_peak_rss_bytes"] = NSNull() }
            finalMetadata["thermal_state"] = ProcessInfo.processInfo.thermalState.rawValue
            finalMetadata["low_power_mode"] = ProcessInfo.processInfo.isLowPowerModeEnabled
            do { try check() } catch { if outcome == "complete" { outcome = "invalid"; reason = "\(error)" } }
            try write(presentations, "metal-observations.json")
            let executable = Bundle.main.executableURL.flatMap { try? Data(contentsOf: $0) }
            let build = Bundle.main.url(forResource: "build", withExtension: "json").flatMap { try? Data(contentsOf: $0) }.flatMap { try? JSONSerialization.jsonObject(with: $0) }
            let manifest: [String: Any] = ["schema_version": 1, "initialization": initialization, "native_context_ctm": nativeCTM, "first_use_definition": "first painter use in fresh process; replay construction/reference oracle and context creation logged separately before replay epoch", "status": outcome, "reason": reason, "backend": o.backend, "policy": o.policy, "scene": o.scene, "trace": o.trace, "trace_source": "synthetic 240Hz, not recorded Wacom input", "duration_s": o.duration, "start_state": o.startState, "warmup_s": o.warmup, "idle_s": o.idle, "logging_enabled": o.logging,
                "occlusion_checks": occlusionChecks, "occlusion_query_unavailable": occlusionUnavailable, "physical_pen_to_screen": "NOT MEASURED — camera pending", "software_endpoint": o.backend == "coregraphics" ? "AppKit drawing callback return; raster and composition deferred/unknown" : "Skia submit and separately reported Metal callbacks",
                "redraw_scheduling": "Common event-driven invalidation with one pending request; Core Graphics NSView.draw callback, Metal explicit main-runloop dispatch into native CAMetalLayer drawable. No display-link pacing or application frame FIFO.", "redraw_watchdog_s": 2, "cache_scheduling": "synchronous in draw; atomic generation switch; input remains queued during rebuild", "initial": initialState, "final": finalMetadata, "lifecycle": events,
                "build": build ?? NSNull(), "binary_sha256": executable.map { SHA256.hash(data: $0).map { String(format: "%02x", $0) }.joined() } ?? "unknown",
                "os": ProcessInfo.processInfo.operatingSystemVersionString, "thermal_state": ProcessInfo.processInfo.thermalState.rawValue, "self_test": o.selfTest,
                "metal_observation_count": presentations.count, "metal_configuration": ["maximum_drawables": 3, "display_sync": true, "framebuffer_only": false], "limitations": ["No photon endpoint", "Generated replay bypasses digitizer/driver", "AppKit occlusion flag does not prove absence of partial occluders", "Final screenshot is untimed", "Metal completion callbacks are not GPU-only durations"]]
            try write(manifest, "manifest.json")
            try require(gpuRetired, "GPU retirement was not proven")
            if let replay { pc_replay_destroy(replay); self.replay = nil }; if let skia { pc_skia_destroy(skia); self.skia = nil }
            if let activity { ProcessInfo.processInfo.endActivity(activity); self.activity = nil }
            finalizing = false; stop.isEnabled = false; results.isEnabled = true; status.stringValue = "\(outcome): \(reason) — physical latency pending"
            // A result directory is immutable; another run requires a fresh invocation.
            run.isEnabled = false
            if closeAfterFinished { NSApp.terminate(nil) }
            if !o.interactive && !o.holdResults { exit(outcome == "complete" || (o.selfTest && outcome == "cancelled") ? 0 : 1) }
        } catch { fputs("Finalization failed: \(error)\n", stderr); exit(1) }
    }
}
let app = NSApplication.shared
if CommandLine.arguments.contains("--list-displays") {
    do { print(String(decoding: try jsonData(screens()), as: UTF8.self)); exit(0) } catch { fputs("\(error)\n", stderr); exit(1) }
}
do {
    let host = Host(try Options.parse()); app.setActivationPolicy(.regular); app.delegate = host; withExtendedLifetime(host) { app.run() }
} catch { fputs("\(error)\n", stderr); exit(1) }
