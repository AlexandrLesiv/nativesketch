import AppKit
import CoreVideo
import CryptoKit
import Darwin
import QuartzCore

let fixtureIDs = [
  "live", "dense100", "dense1000", "dense5000", "panzoom", "alpha", "clip", "image",
]
enum Failure: Error { case message(String) }
func fail(_ text: String) -> Failure { .message(text) }
func ffiError() -> Failure { fail(String(cString: rc_error(nil))) }
func now() -> Double { CACurrentMediaTime() }
func systemText(_ key: String) -> Any {
  var size = 0
  guard sysctlbyname(key, nil, &size, nil, 0) == 0, size > 0 else { return NSNull() }
  var bytes = [CChar](repeating: 0, count: size)
  guard sysctlbyname(key, &bytes, &size, nil, 0) == 0 else { return NSNull() }
  return String(cString: bytes)
}
func doubles<T>(_ tuple: T) -> [Double] {
  withUnsafeBytes(of: tuple) { Array($0.bindMemory(to: Double.self)) }
}
func affine<T>(_ tuple: T) -> CGAffineTransform {
  let a = doubles(tuple)
  return CGAffineTransform(a: a[0], b: a[1], c: a[2], d: a[3], tx: a[4], ty: a[5])
}
func digest(_ data: Data) -> String {
  SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined()
}
struct Options {
  var backend = "coregraphics", mode = "appkit-paced", fixture = "live", width = 1600,
    height = 1000, warmup = 120, frames = 600, repeatID = "0", output = ""
  var duration: Double?, interactive = false
  static func parse() throws -> Options {
    var o = Options()
    let args = Array(CommandLine.arguments.dropFirst())
    var i = 0
    var seen = Set<String>()
    o.interactive = args.isEmpty || args.contains("--interactive")
    while i < args.count {
      let key = args[i]
      i += 1
      if key == "--interactive" { continue }
      guard i < args.count, seen.insert(key).inserted else {
        throw fail("Missing or duplicate option \(key)")
      }
      let v = args[i]
      i += 1
      func integer() throws -> Int {
        guard let n = Int(v), n >= 0 else { throw fail("Invalid integer \(key)") }
        return n
      }
      switch key {
      case "--backend": o.backend = v
      case "--mode": o.mode = v
      case "--fixture": o.fixture = v
      case "--width": o.width = try integer()
      case "--height": o.height = try integer()
      case "--warmup": o.warmup = try integer()
      case "--frames": o.frames = try integer()
      case "--repeat-id": o.repeatID = v
      case "--output":
        guard v.hasPrefix("/") else { throw fail("Output must be absolute") }
        o.output = v
      case "--duration-seconds":
        guard let d = Double(v), d.isFinite, d > 0, d <= 14400 else {
          throw fail("Invalid duration")
        }
        o.duration = d
      default: throw fail("Unknown option \(key)")
      }
    }
    guard o.backend == "coregraphics", o.mode == "appkit-paced", fixtureIDs.contains(o.fixture),
      (64...8192).contains(o.width), (64...8192).contains(o.height), o.frames > 0,
      o.frames <= 1_000_000, o.warmup <= 100000, !o.repeatID.isEmpty,
      !(seen.contains("--frames") && o.duration != nil)
    else { throw fail("Invalid options") }
    if o.output.isEmpty {
      o.output =
        FileManager.default.temporaryDirectory.appendingPathComponent(
          "coregraphics-" + UUID().uuidString
        ).path
    }
    return o
  }
}
struct Row {
  let phase: String, index: UInt64, stateHash: String
  let request: Double, start: Double, end: Double, input: Double, outline: Double, export: Double,
    path: Double, draw: Double, service: Double
  var json: [String: Any] {
    [
      "phase": phase, "index": index, "state_hash": stateHash, "request_s": request,
      "draw_start_s": start, "draw_end_s": end, "input_ms": input, "outline_ms": outline,
      "export_ms": export, "path_ms": path, "draw_ms": draw, "service_ms": service,
    ]
  }
  var csv: String {
    [phase, String(index), stateHash].joined(separator: ",") + ","
      + [request, start, end, input, outline, export, path, draw, service].map {
        String(format: "%.9f", $0)
      }.joined(separator: ",")
  }
}
final class PulseGate {
  let lock = NSLock()
  var pending = false, stopped = true
  var opportunities = 0, coalesced = 0, requests = 0
  func request() -> Bool {
    lock.lock()
    defer { lock.unlock() }
    if stopped { return false }
    opportunities += 1
    if pending {
      coalesced += 1
      return false
    }
    pending = true
    requests += 1
    return true
  }
  func release() {
    lock.lock()
    pending = false
    lock.unlock()
  }
  func reset() {
    lock.lock()
    pending = false
    stopped = true
    opportunities = 0
    coalesced = 0
    requests = 0
    lock.unlock()
  }
  func setStopped(_ value: Bool) {
    lock.lock()
    stopped = value
    lock.unlock()
  }
  func counts() -> [String: Any] {
    lock.lock()
    defer { lock.unlock() }
    return ["opportunities": opportunities, "coalesced": coalesced, "requests": requests]
  }
}
final class Canvas: NSView {
  var ticket: ((CGContext) -> Void)?, painter: ((CGContext) -> Void)?
  var benchmarkDraws = 0, unsolicitedDraws = 0
  override var isFlipped: Bool { true }
  override var isOpaque: Bool { true }
  override func draw(_ dirtyRect: NSRect) {
    guard let c = NSGraphicsContext.current?.cgContext else { return }
    if let work = ticket {
      ticket = nil
      benchmarkDraws += 1
      work(c)
    } else {
      unsolicitedDraws += 1
      if let painter {
        painter(c)
      } else {
        c.setFillColor(CGColor(gray: 1, alpha: 1))
        c.fill(bounds)
      }
    }
  }
}
final class Host: NSObject, NSApplicationDelegate, NSWindowDelegate {
  var options: Options
  let canvas = Canvas(),
    status = NSTextField(wrappingLabelWithString: "Ready — native AppKit / Core Graphics"),
    run = NSButton(title: "Run", target: nil, action: nil),
    stop = NSButton(title: "Stop", target: nil, action: nil)
  var window: NSWindow!, link: CVDisplayLink?, handle: UnsafeMutableRawPointer?,
    activity: NSObjectProtocol?
  let gate = PulseGate(),
    activityOptions: ProcessInfo.ActivityOptions = [
      .userInitiated, .latencyCritical, .idleDisplaySleepDisabled,
    ]
  var active = false, finishing = false, exitCode: Int32 = 0, phase = "first-use",
    index: UInt64 = 0, measuredBegin: Double?, measuredElapsed = 0.0
  var rows: [Row] = [], manifest: [String: Any] = [:], metadata: [String: Any] = [:],
    lifecycleStart: [String: Any] = [:], events: [[String: Any]] = [],
    observations: [NSObjectProtocol] = []
  var paths: [(CGPath, CGColor)] = [], clip: CGPath?, image: CGImage?, frame = RcFixtureFrame(),
    ctm: [Double] = [], outputValid = false
  var sessionResigned = false, sleeping = false, displaySleeping = false
  let colorSpace = CGColorSpace(name: CGColorSpace.sRGB)!
  init(_ o: Options) { options = o }
  func write(_ object: Any, _ name: String) throws {
    let data = try JSONSerialization.data(
      withJSONObject: object, options: [.prettyPrinted, .sortedKeys])
    try data.write(
      to: URL(fileURLWithPath: options.output).appendingPathComponent(name), options: .atomic)
  }
  func log(_ text: String) {
    let url = URL(fileURLWithPath: options.output).appendingPathComponent("run.log")
    let bytes = Data("\(now()) \(text)\n".utf8)
    if let f = try? FileHandle(forWritingTo: url) {
      _ = try? f.seekToEnd()
      try? f.write(contentsOf: bytes)
      try? f.close()
    } else {
      try? bytes.write(to: url)
    }
  }
  func state() -> [String: Any] {
    let session = CGSessionCopyCurrentDictionary() as? [String: Any]
    let display =
      (window.screen?.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber)?
      .uint32Value ?? CGMainDisplayID()
    return [
      "window_visible": window.isVisible,
      "window_unoccluded": window.occlusionState.contains(.visible),
      "window_miniaturized": window.isMiniaturized, "window_level": window.level.rawValue,
      "session_on_console": (session?[kCGSessionOnConsoleKey as String] as? NSNumber)?.boolValue
        ?? false, "session_resigned": sessionResigned, "system_sleeping": sleeping,
      "screens_sleeping": displaySleeping, "display_asleep": CGDisplayIsAsleep(display) != 0,
      "system_uptime_seconds": ProcessInfo.processInfo.systemUptime,
    ]
  }
  func eligible() -> Bool {
    let s = state()
    return ["window_visible", "window_unoccluded", "session_on_console"].allSatisfy {
      s[$0] as? Bool == true
    }
      && [
        "window_miniaturized", "session_resigned", "system_sleeping", "screens_sleeping",
        "display_asleep",
      ].allSatisfy { s[$0] as? Bool == false }
  }
  func physical() -> [String: Any] {
    let b = canvas.convertToBacking(canvas.bounds)
    return [
      "backing_width": b.width, "backing_height": b.height, "point_width": canvas.bounds.width,
      "point_height": canvas.bounds.height, "scale": window.backingScaleFactor,
      "backing_origin": [b.minX, b.minY], "ctm": ctm,
    ]
  }
  func checkDimensions() throws {
    let b = canvas.convertToBacking(canvas.bounds)
    guard abs(b.width - Double(options.width)) < 0.01,
      abs(b.height - Double(options.height)) < 0.01, abs(b.minX.rounded() - b.minX) < 0.01,
      abs(b.minY.rounded() - b.minY) < 0.01
    else { throw fail("Native canvas backing dimensions/alignment differ from requested raster") }
    guard let screen = window.screen,
      screen.visibleFrame.insetBy(dx: -0.5, dy: -0.5).contains(window.frame)
    else { throw fail("Requested native canvas and controls do not fit the visible screen") }
  }
  func applicationDidFinishLaunching(_ n: Notification) {
    let menu = NSMenu()
    let item = NSMenuItem()
    let appMenu = NSMenu()
    appMenu.addItem(
      withTitle: "Quit Core Graphics Comparison", action: #selector(NSApplication.terminate(_:)),
      keyEquivalent: "q")
    item.submenu = appMenu
    menu.addItem(item)
    NSApp.mainMenu = menu
    let screen = NSScreen.main!
    let scale = screen.backingScaleFactor
    let w = Double(options.width) / scale
    let h = Double(options.height) / scale
    window = NSWindow(
      contentRect: NSRect(x: 0, y: 0, width: max(w + 28, 600), height: h + 110),
      styleMask: [.titled, .closable, .miniaturizable], backing: .buffered, defer: false)
    window.title = "Core Graphics — \(options.fixture) — \(options.width) × \(options.height)"
    window.delegate = self
    window.colorSpace = NSColorSpace.sRGB
    if !options.interactive { window.level = .floating }
    let content = NSView()
    window.contentView = content
    canvas.frame = NSRect(x: 14, y: 14, width: w, height: h)
    canvas.wantsLayer = true
    canvas.layerContentsRedrawPolicy = .onSetNeedsDisplay
    canvas.setAccessibilityElement(true)
    canvas.setAccessibilityLabel("Deterministic Core Graphics preview; no drawing input")
    status.frame = NSRect(x: 14, y: h + 20, width: max(w - 200, 250), height: 65)
    status.setAccessibilityLabel("Experiment status")
    run.frame = NSRect(x: max(w - 180, 350), y: h + 42, width: 80, height: 32)
    stop.frame = NSRect(x: max(w - 90, 440), y: h + 42, width: 80, height: 32)
    run.target = self
    run.action = #selector(start)
    stop.target = self
    stop.action = #selector(cancel)
    stop.keyEquivalent = "."
    stop.keyEquivalentModifierMask = .command
    stop.isEnabled = false
    [canvas, status, run, stop].forEach { content.addSubview($0) }
    for name in [
      NSWorkspace.sessionDidResignActiveNotification,
      NSWorkspace.sessionDidBecomeActiveNotification, NSWorkspace.willSleepNotification,
      NSWorkspace.didWakeNotification, NSWorkspace.screensDidSleepNotification,
      NSWorkspace.screensDidWakeNotification,
    ] {
      observations.append(
        NSWorkspace.shared.notificationCenter.addObserver(forName: name, object: nil, queue: .main)
        { [weak self] note in self?.workspace(note) })
    }
    canvas.painter = { [weak self] context in self?.paint(context) }
    window.center()
    window.makeKeyAndOrderFront(nil)
    NSApp.activate(ignoringOtherApps: true)
    if !options.interactive { DispatchQueue.main.asyncAfter(deadline: .now() + 2) { self.start() } }
  }
  func workspace(_ n: Notification) {
    switch n.name {
    case NSWorkspace.sessionDidResignActiveNotification: sessionResigned = true
    case NSWorkspace.sessionDidBecomeActiveNotification: sessionResigned = false
    case NSWorkspace.willSleepNotification: sleeping = true
    case NSWorkspace.didWakeNotification: sleeping = false
    case NSWorkspace.screensDidSleepNotification: displaySleeping = true
    case NSWorkspace.screensDidWakeNotification: displaySleeping = false
    default: break
    }
    if sessionResigned || sleeping || displaySleeping { invalidate(n.name.rawValue) }
  }
  func invalidate(_ reason: String) {
    guard active else { return }
    events.append(["event": reason, "invalidating": true, "state": state()])
    finish("invalidated", reason)
  }
  func windowDidMiniaturize(_ n: Notification) { invalidate("Window minimized") }
  func windowDidResize(_ n: Notification) { invalidate("Window resized") }
  func windowDidChangeScreen(_ n: Notification) { invalidate("Display changed") }
  func windowDidChangeBackingProperties(_ n: Notification) { invalidate("Backing scale changed") }
  func windowDidChangeOcclusionState(_ n: Notification) {
    if active {
      events.append([
        "event": "Window occlusion changed", "invalidating": !eligible(), "state": state(),
      ])
      if !eligible() { invalidate("Window fully occluded") }
    }
  }
  func windowShouldClose(_ sender: NSWindow) -> Bool {
    if active { finish("cancelled", "Window closed") }
    NSApp.terminate(nil)
    return false
  }
  func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
    if active { finish("cancelled", "Quit requested") }
    return .terminateNow
  }
  func applicationWillTerminate(_ n: Notification) {
    fflush(nil)
    exit(exitCode)
  }
  @objc func cancel() { finish("cancelled", "Stop requested") }
  @objc func start() {
    guard !active else { return }
    if FileManager.default.fileExists(atPath: options.output + "/manifest.json")
      || FileManager.default.fileExists(atPath: options.output + "/host-status.json")
    {
      if options.interactive {
        options.output += "-" + UUID().uuidString
      } else {
        exitCode = 1
        status.stringValue = "Output already contains results; preserved"
        fputs("Output already contains results; preserved\n", stderr)
        NSApp.terminate(nil)
        return
      }
    }
    phase = "first-use"
    index = 0
    measuredBegin = nil
    measuredElapsed = 0
    rows = []
    manifest = [:]
    metadata = [:]
    events = []
    paths = []
    clip = nil
    image = nil
    frame = RcFixtureFrame()
    ctm = []
    outputValid = false
    canvas.benchmarkDraws = 0
    canvas.unsolicitedDraws = 0
    gate.reset()
    do {
      try FileManager.default.createDirectory(
        atPath: options.output, withIntermediateDirectories: true)
      try checkDimensions()
      guard eligible() else { throw fail("Visible unoccluded window and active console required") }
      lifecycleStart = state()
      active = true
      run.isEnabled = false
      stop.isEnabled = true
      activity = ProcessInfo.processInfo.beginActivity(
        options: activityOptions, reason: "Core Graphics benchmark through output validation")
      let t = now()
      handle = options.fixture.withCString {
        rc_fixture_create($0, UInt32(options.width), UInt32(options.height))
      }
      guard handle != nil, let p = rc_fixture_metadata_json(handle) else { throw ffiError() }
      metadata =
        try JSONSerialization.jsonObject(with: Data(String(cString: p).utf8)) as! [String: Any]
      let screenID =
        (window.screen?.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber)?
        .uint32Value ?? CGMainDisplayID()
      guard CVDisplayLinkCreateWithCGDisplay(screenID, &link) == kCVReturnSuccess, let link else {
        throw fail("Display link creation failed")
      }
      let period = CVDisplayLinkGetNominalOutputVideoRefreshPeriod(link)
      var build: [String: Any] = [:]
      if let u = Bundle.main.url(forResource: "build", withExtension: "json") {
        build = (try JSONSerialization.jsonObject(with: Data(contentsOf: u))) as! [String: Any]
      }
      manifest = metadata
      manifest.merge(
        [
          "schema_version": 2, "experiment": "coregraphics-appkit-v1", "backend": options.backend,
          "mode": options.mode, "fixture": options.fixture, "width": options.width,
          "height": options.height, "warmup": options.warmup,
          "frames_requested": options.duration == nil ? options.frames as Any : NSNull(),
          "duration_seconds": options.duration as Any? ?? NSNull(), "repeat_id": options.repeatID,
          "output": options.output, "status": "running", "build": build,
          "init_ms": (now() - t) * 1000,
          "display": [
            "refresh_hz": period.timeValue > 0 && period.flags == 0
              ? Double(period.timeScale) / Double(period.timeValue) as Any : NSNull(),
            "refresh_source": "CVDisplayLink nominal period, not observed presentation",
            "backing_scale": window.backingScaleFactor,
          ],
          "runtime": [
            "os": ProcessInfo.processInfo.operatingSystemVersionString,
            "cpu_count": ProcessInfo.processInfo.processorCount,
            "cpu_model": systemText("machdep.cpu.brand_string"),
            "hardware_model": systemText("hw.model"),
            "os_build": systemText("kern.osversion"),
            "physical_memory_bytes": ProcessInfo.processInfo.physicalMemory,
          ],
          "power": [
            "thermal_before": ProcessInfo.processInfo.thermalState.rawValue,
            "low_power_mode": ProcessInfo.processInfo.isLowPowerModeEnabled,
          ],
          "quality": [
            "target": "layer-backed NSView.draw native CGContext",
            "alpha": "straight RGBA source; CGContext source-over",
            "fill": "nonzero once per complete stroke",
            "color_space": "explicit sRGB colors and image; sRGB window",
            "antialiasing": "CGContext allowsAntialiasing and shouldAntialias true",
            "image_filter":
              "CGInterpolationQuality.low requested bilinear; implementation-specific",
            "cache_policy": "retained CGPaths; fresh full-view raster; live paths rebuilt",
          ],
          "timing_provenance": [
            "clock": "CACurrentMediaTime monotonic seconds; Rust Instant stage durations",
            "service_ms":
              "NSView.draw entry through fixture preparation, CGPath construction and Core Graphics paint calls; excludes state hashing/output; draw return may exclude deferred compositor work",
            "draw_ms": "nested Core Graphics painting body only",
            "scheduling":
              "CVDisplayLink opportunities; at most one pending main-thread request; no trace advance on coalesced opportunities",
            "request_to_draw": "display-link scheduling diagnostic, not physical input latency",
          ],
          "unsupported": [
            "gpu_execution_ms": "Native CGContext exposes no equivalent measurement",
            "gpu_completion": "Not observable",
            "presentation_timestamp": "Draw return is not presentation",
            "presentation_deadline_misses": "No independent presentation schedule",
            "gpu_peak_memory": "No native comparable counter",
            "physical_footprint": "Not implemented",
            "physical_pen_latency": "No instrumented physical pen measurement",
          ],
        ], uniquingKeysWith: { $1 })
      try write(manifest, "manifest.json")
      log("START \(options.fixture) \(options.width)x\(options.height)")
      let result = CVDisplayLinkSetOutputCallback(
        link,
        { _, _, _, _, _, context in
          guard let context else { return kCVReturnError }
          let host = Unmanaged<Host>.fromOpaque(context).takeUnretainedValue()
          if host.gate.request() {
            let request = now()
            DispatchQueue.main.async {
              host.tick(request)
              host.gate.release()
            }
          }
          return kCVReturnSuccess
        }, Unmanaged.passUnretained(self).toOpaque())
      guard result == kCVReturnSuccess else { throw fail("Cannot install display-link callback") }
      gate.setStopped(false)
      guard CVDisplayLinkStart(link) == kCVReturnSuccess else {
        throw fail("Cannot start display link")
      }
      status.stringValue = "Measuring \(options.fixture) · actual NSView.draw\n\(options.output)"
    } catch { finish("failed", String(describing: error)) }
  }
  func makePath(_ commands: UnsafePointer<RcCommand>, _ start: Int, _ count: Int) -> CGPath {
    let path = CGMutablePath()
    for i in start..<(start + count) {
      let c = commands[i]
      let p = doubles(c.points)
      switch c.verb {
      case 0: path.move(to: CGPoint(x: p[0], y: p[1]))
      case 1: path.addLine(to: CGPoint(x: p[0], y: p[1]))
      case 2:
        path.addCurve(
          to: CGPoint(x: p[4], y: p[5]), control1: CGPoint(x: p[0], y: p[1]),
          control2: CGPoint(x: p[2], y: p[3]))
      default: path.closeSubpath()
      }
    }
    return path
  }
  func prepare(_ index: UInt64) throws {
    guard rc_fixture_prepare(handle, index, &frame) == 0 else { throw ffiError() }
  }
  func buildPaths() throws {
    if frame.geometry_changed != 0 {
      guard let commands = frame.commands, let outlines = frame.outlines else {
        throw fail("Missing fixture buffers")
      }
      paths = (0..<Int(frame.outline_count)).map { i in
        let p = outlines[i]
        return (
          makePath(commands, Int(p.command_start), Int(p.command_count)),
          CGColor(colorSpace: colorSpace, components: doubles(p.rgba).map { CGFloat($0) })!
        )
      }
    }
    if frame.has_clip != 0 && clip == nil, let c = frame.clip_commands {
      clip = makePath(c, 0, Int(frame.clip_command_count))
    }
    if options.fixture == "image" && image == nil {
      var count: UInt64 = 0
      guard let bytes = rc_fixture_image_rgba(handle, &count), count == 2048 * 2048 * 4 else {
        throw ffiError()
      }
      let data = Data(bytes: bytes, count: Int(count))
      guard let provider = CGDataProvider(data: data as CFData),
        let value = CGImage(
          width: 2048, height: 2048, bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: 8192,
          space: colorSpace, bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.last.rawValue),
          provider: provider, decode: nil, shouldInterpolate: true, intent: .defaultIntent)
      else { throw fail("Cannot create fixture CGImage") }
      image = value
    }
  }
  func paint(_ c: CGContext) {
    c.saveGState()
    defer { c.restoreGState() }
    c.setBlendMode(.copy)
    c.setFillColor(CGColor(colorSpace: colorSpace, components: [1, 1, 1, 1])!)
    c.fill(canvas.bounds)
    c.setBlendMode(.normal)
    c.setAllowsAntialiasing(true)
    c.setShouldAntialias(true)
    c.interpolationQuality = .low
    c.scaleBy(x: 1 / window.backingScaleFactor, y: 1 / window.backingScaleFactor)
    c.concatenate(affine(frame.transform))
    if frame.has_clip != 0, let clip {
      c.clip(to: CGRect(x: 150, y: 130, width: 1300, height: 740))
      c.addPath(clip)
      c.clip(using: .winding)
    }
    if let image {
      c.saveGState()
      c.concatenate(affine(frame.image_transform))
      c.translateBy(x: 0, y: 2048)
      c.scaleBy(x: 1, y: -1)
      c.draw(image, in: CGRect(x: 0, y: 0, width: 2048, height: 2048))
      c.restoreGState()
    }
    for (path, color) in paths {
      c.setFillColor(color)
      c.addPath(path)
      c.fillPath(using: .winding)
    }
  }
  func hash(_ index: UInt64, _ actual: UInt32) throws -> String {
    guard let p = rc_fixture_state_hash(handle, index, actual) else { throw ffiError() }
    return String(cString: p)
  }
  func tick(_ request: Double) {
    guard active && !finishing else { return }
    do {
      guard eligible() else { throw fail("Lifecycle guard failed before draw") }
      try checkDimensions()
      if phase == "measured" && measuredBegin == nil { measuredBegin = now() }
      let p = phase
      let i = index
      canvas.ticket = { context in
        let start = now()
        do {
          try self.prepare(i)
          let pathStart = now()
          try self.buildPaths()
          let drawStart = now()
          self.paint(context)
          let end = now()
          if self.ctm.isEmpty {
            let m = context.ctm
            self.ctm = [m.a, m.b, m.c, m.d, m.tx, m.ty]
            let s = self.window.backingScaleFactor
            guard abs(abs(m.a) - s) < 0.01, abs(abs(m.d) - s) < 0.01, abs(m.b) < 0.01,
              abs(m.c) < 0.01, abs(m.tx.rounded() - m.tx) < 0.01, abs(m.ty.rounded() - m.ty) < 0.01
            else { throw fail("Unexpected native drawing-context scale/alignment") }
          }
          let identity = try self.hash(i, 0)
          self.rows.append(
            Row(
              phase: p, index: i, stateHash: identity, request: request, start: start, end: end,
              input: self.frame.input_ms, outline: self.frame.outline_ms,
              export: self.frame.export_ms, path: (drawStart - pathStart) * 1000,
              draw: (end - drawStart) * 1000, service: (end - start) * 1000))
        } catch { self.finish("failed", String(describing: error)) }
      }
      canvas.needsDisplay = true
      canvas.displayIfNeeded()
      guard canvas.ticket == nil else {
        canvas.ticket = nil
        throw fail("Requested redraw did not invoke NSView.draw")
      }
      guard active else { return }
      if p == "first-use" {
        phase = options.warmup == 0 ? "measured" : "warmup"
        index = 0
      } else if p == "warmup" {
        index += 1
        if index >= UInt64(options.warmup) {
          phase = "measured"
          index = 0
        }
      } else {
        index += 1
        measuredElapsed = now() - (measuredBegin ?? now())
        if options.duration.map({ measuredElapsed >= $0 }) ?? (index >= UInt64(options.frames)) {
          finish("complete", "")
        }
      }
    } catch { finish("invalidated", String(describing: error)) }
  }
  func capture(_ captureIndex: UInt64) throws {
    let expected = try hash(captureIndex, 0)
    let actual = try hash(captureIndex, 1)
    guard expected == actual else { throw fail("Capture fixture identity mismatch") }
    guard let rep = canvas.bitmapImageRepForCachingDisplay(in: canvas.bounds) else {
      throw fail("No cacheDisplay bitmap")
    }
    canvas.cacheDisplay(in: canvas.bounds, to: rep)
    guard rep.pixelsWide == options.width, rep.pixelsHigh == options.height, let cg = rep.cgImage
    else { throw fail("Capture raster size mismatch") }
    var rgba = [UInt8](repeating: 0, count: options.width * options.height * 4)
    let normalized: CGImage? = rgba.withUnsafeMutableBytes { bytes in
      guard
        let c = CGContext(
          data: bytes.baseAddress, width: options.width, height: options.height,
          bitsPerComponent: 8, bytesPerRow: options.width * 4, space: colorSpace,
          bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
            | CGBitmapInfo.byteOrder32Big.rawValue)
      else { return nil }
      c.draw(cg, in: CGRect(x: 0, y: 0, width: options.width, height: options.height))
      return c.makeImage()
    }
    guard let normalized,
      let png = NSBitmapImageRep(cgImage: normalized).representation(using: .png, properties: [:])
    else { throw fail("PNG encoding failed") }
    try png.write(to: URL(fileURLWithPath: options.output + "/final.png"), options: .atomic)
    try write(
      [
        "width": options.width, "height": options.height, "frame_index": captureIndex,
        "rgba_sha256": digest(Data(rgba)), "png_sha256": digest(png), "state_hash": expected,
        "actual_state_hash": actual, "fixture_hash": metadata["fixture_hash"]!,
        "input_hash": metadata["input_hash"]!, "image_hash": metadata["image_hash"]!,
        "provenance":
          "Untimed NSView.cacheDisplay of final measured frame; normalized opaque sRGB RGBA8; not proof of exact displayed pixels",
      ], "readback.json")
  }
  func finish(_ supplied: String, _ suppliedReason: String) {
    guard !finishing else { return }
    finishing = true
    gate.setStopped(true)
    if let link { CVDisplayLinkStop(link) }
    canvas.ticket = nil
    var result = supplied
    var reason = suppliedReason
    do {
      try FileManager.default.createDirectory(
        atPath: options.output, withIntermediateDirectories: true)
      if result == "complete" {
        guard eligible() else { throw fail("Lifecycle invalid at output start") }
        try checkDimensions()
        guard let last = rows.last(where: { $0.phase == "measured" }) else {
          throw fail("No measured frames")
        }
        let actual = try hash(last.index, 1)
        guard last.stateHash == actual else {
          throw fail("Final measured geometry identity mismatch")
        }
        manifest["final_expected_state_hash"] = last.stateHash
        manifest["final_actual_state_hash"] = actual
        try capture(last.index)
        try checkDimensions()
        guard eligible() else { throw fail("Lifecycle invalid after capture") }
        outputValid = true
      }
      let header =
        "phase,index,state_hash,request_s,draw_start_s,draw_end_s,input_ms,outline_ms,export_ms,path_ms,draw_ms,service_ms\n"
      try Data((header + rows.map { $0.csv }.joined(separator: "\n") + "\n").utf8).write(
        to: URL(fileURLWithPath: options.output + "/frames.csv"), options: .atomic)
    } catch {
      result = "failed"
      reason += " \(error)"
      outputValid = false
    }
    manifest["status"] = result
    manifest["error"] = reason
    manifest["completed_sample_count"] = rows.filter { $0.phase == "measured" }.count
    manifest["measured_elapsed_seconds"] = measuredElapsed
    manifest["first_use"] = rows.first(where: { $0.phase == "first-use" })?.json ?? [:]
    manifest["physical_canvas"] = physical()
    var counts = gate.counts()
    counts["benchmark_draw_callbacks"] = canvas.benchmarkDraws
    counts["unsolicited_draw_callbacks"] = canvas.unsolicitedDraws
    manifest["scheduling"] = counts
    var usage = rusage()
    getrusage(RUSAGE_SELF, &usage)
    manifest["os_peak_rss_bytes"] = usage.ru_maxrss
    do {
      try write(manifest, "manifest.json")
      log("FINISH \(result) \(reason)")
      // Retain assertion through all evidence writes; sample lifecycle at the final write.
      if result == "complete" && !eligible() {
        result = "invalidated"
        reason = "Lifecycle changed during output"
        outputValid = false
      }
      try write(
        [
          "status": result, "reason": reason, "lifecycle_start": lifecycleStart,
          "lifecycle_end": state(), "lifecycle_events": events,
          "activity_assertion": [
            "held_through_output": activity != nil,
            "options": ["userInitiated", "latencyCritical", "idleDisplaySleepDisabled"],
          ], "output_validation": ["valid": outputValid, "reason": reason],
          "final_dimensions": [
            "width": canvas.convertToBacking(canvas.bounds).width,
            "height": canvas.convertToBacking(canvas.bounds).height,
          ], "thermal_after": ProcessInfo.processInfo.thermalState.rawValue,
        ], "host-status.json")
    } catch {
      result = "failed"
      reason += " Output write failed: \(error)"
    }
    if let activity {
      ProcessInfo.processInfo.endActivity(activity)
      self.activity = nil
    }
    if let handle {
      rc_fixture_destroy(handle)
      self.handle = nil
    }
    active = false
    window.level = .normal
    run.isEnabled = true
    stop.isEnabled = false
    exitCode = result == "complete" ? 0 : 1
    status.stringValue = "\(result.capitalized): \(reason)\n\(options.output)"
    print("\(result): \(options.output) \(reason)")
    fflush(nil)
    if !options.interactive { NSApp.terminate(nil) }
    finishing = false
  }
}
do {
  let o = try Options.parse()
  let app = NSApplication.shared
  app.setActivationPolicy(.regular)
  let host = Host(o)
  app.delegate = host
  app.run()
  exit(host.exitCode)
} catch {
  fputs("CoreGraphicsComparison: \(error)\n", stderr)
  exit(2)
}
