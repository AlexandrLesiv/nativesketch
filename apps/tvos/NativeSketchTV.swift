import Darwin
import UIKit
import Foundation

// Delegate streaming bounds memory before appending; redirects cannot downgrade HTTPS.
final class DocumentDownload: NSObject, URLSessionDataDelegate, @unchecked Sendable {
  static let limit = 64 * 1024 * 1024
  private var body = BoundedDownload(limit: DocumentDownload.limit)
  private let sessionLock = NSLock()
  private var session: URLSession?
  private let completion: @Sendable (Result<Data, Error>) -> Void
  init(url: URL, completion: @escaping @Sendable (Result<Data, Error>) -> Void) {
    self.completion = completion
    super.init()
    let configuration = URLSessionConfiguration.ephemeral
    configuration.timeoutIntervalForResource = 120
    let queue = OperationQueue(); queue.maxConcurrentOperationCount = 1
    let created = URLSession(configuration: configuration, delegate: self, delegateQueue: queue)
    sessionLock.withLock { session = created }
    created.dataTask(with: url).resume()
  }
  func cancel() {
    let current = sessionLock.withLock { session }
    current?.invalidateAndCancel()
  }
  func urlSession(_ session: URLSession, task: URLSessionTask, willPerformHTTPRedirection response: HTTPURLResponse, newRequest request: URLRequest, completionHandler: @escaping @Sendable (URLRequest?) -> Void) {
    completionHandler(body.redirect(to: request.url) ? request : nil)
  }
  func urlSession(_ session: URLSession, dataTask: URLSessionDataTask, didReceive response: URLResponse, completionHandler: @escaping @Sendable (URLSession.ResponseDisposition) -> Void) {
    guard body.response(status: (response as? HTTPURLResponse)?.statusCode, expectedLength: response.expectedContentLength) else {
      completionHandler(.cancel); return
    }
    completionHandler(.allow)
  }
  func urlSession(_ session: URLSession, dataTask: URLSessionDataTask, didReceive data: Data) {
    if !body.append(data) { dataTask.cancel() }
  }
  func urlSession(_ session: URLSession, task: URLSessionTask, didCompleteWithError error: Error?) {
    completion(body.result(transportError: error))
    session.finishTasksAndInvalidate()
    sessionLock.withLock { self.session = nil }
  }
}

@MainActor final class TVCanvas: UIView {
  var document: ReadOnlyDocument
  var navigation = ViewerNavigation() { didSet { setNeedsDisplay() } }
  init(_ document: ReadOnlyDocument) {
    self.document = document; super.init(frame: .zero)
    backgroundColor = .white; isAccessibilityElement = true
    accessibilityLabel = "Drawing preview, view only"
  }
  required init?(coder: NSCoder) { fatalError() }
  override func draw(_ rect: CGRect) {
    guard let context = UIGraphicsGetCurrentContext() else { return }
    context.concatenate(navigation.transform(bounds: document.bounds, size: bounds.size))
    for (path, color) in document.paths { context.addPath(path); context.setFillColor(color); context.fillPath() }
  }
}

@MainActor final class TVController: UIViewController {
  var document = ReadOnlyDocument()
  lazy var canvas = TVCanvas(document)
  let status = UILabel()
  let address = UITextField()
  private var download: DocumentDownload?
  private var generation = 0
  private var cancelButton: UIButton?
  private var navigationButtons: [UIButton] = []
  private let cache = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)[0].appendingPathComponent("LastCopy.nsketch")
  override func viewDidLoad() {
    super.viewDidLoad(); view.backgroundColor = UIColor { $0.userInterfaceStyle == .dark ? .black : .white }
    status.text = "Native Sketch · View Only · Open an HTTPS document or Sample"
    status.numberOfLines = 2
    address.placeholder = "https://…/drawing.nsketch"; address.keyboardType = .URL
    address.autocorrectionType = .no; address.autocapitalizationType = .none
    address.accessibilityLabel = "HTTPS document URL"
    address.borderStyle = .roundedRect
    let open = button("Open URL") { [weak self] in self?.openURL() }
    let cancel = button("Cancel Load") { [weak self] in self?.cancelLoad() }
    cancelButton = cancel
    let sample = button("Sample") { [weak self] in self?.sample() }
    let fit = button("Fit") { [weak self] in self?.canvas.navigation.fit() }
    let plus = button("Zoom +") { [weak self] in self?.canvas.navigation.magnify(1.4) }
    let minus = button("Zoom −") { [weak self] in self?.canvas.navigation.magnify(1 / 1.4) }
    let controls = UIStackView(arrangedSubviews: [open, cancel, sample, fit, plus, minus])
    controls.spacing = 24; controls.distribution = .fillEqually
    let directions = UIStackView(arrangedSubviews: [
      button("Pan Left") { [weak self] in self?.canvas.navigation.pan.width -= 80 },
      button("Pan Right") { [weak self] in self?.canvas.navigation.pan.width += 80 },
      button("Pan Up") { [weak self] in self?.canvas.navigation.pan.height -= 80 },
      button("Pan Down") { [weak self] in self?.canvas.navigation.pan.height += 80 },
    ])
    directions.spacing = 24; directions.distribution = .fillEqually
    navigationButtons = [fit, plus, minus] + directions.arrangedSubviews.compactMap { $0 as? UIButton }
    let stack = UIStackView(arrangedSubviews: [status, address, controls, canvas, directions])
    stack.axis = .vertical; stack.spacing = 20; stack.translatesAutoresizingMaskIntoConstraints = false
    view.addSubview(stack)
    NSLayoutConstraint.activate([stack.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 24), stack.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -24), stack.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 12), stack.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor, constant: -12), address.heightAnchor.constraint(equalToConstant: 70), controls.heightAnchor.constraint(equalToConstant: 70), directions.heightAnchor.constraint(equalToConstant: 70)])
    if let data = try? boundedRead(cache, limit: DocumentDownload.limit), (try? document.importCopy(data, limit: DocumentDownload.limit)) != nil { status.text = "Last viewed copy · View Only" }
    refreshAvailability()
    #if DEBUG
    if ProcessInfo.processInfo.arguments.contains("--integration-test") {
      do { try ViewerReplay.run(); print("APPLE VIEWER REPLAY PASS"); exit(0) }
      catch { print("APPLE VIEWER REPLAY FAIL: \(error)"); exit(1) }
    }
    #endif
  }
  private func button(_ title: String, _ action: @escaping @MainActor () -> Void) -> UIButton {
    let button = UIButton(type: .system); button.setTitle(title, for: .normal)
    button.addAction(UIAction { _ in action() }, for: .primaryActionTriggered); return button
  }
  private func refreshAvailability() {
    cancelButton?.isEnabled = download != nil
    for button in navigationButtons { button.isEnabled = !document.paths.isEmpty }
  }
  private func cancelLoad() {
    generation += 1; download?.cancel(); download = nil
    status.text = "Load cancelled · Previous copy retained · View Only"
    refreshAvailability()
  }
  private func present(_ data: Data, title: String) throws {
    let candidate = ReadOnlyDocument(); try candidate.importCopy(data, limit: DocumentDownload.limit)
    try FileManager.default.createDirectory(at: cache.deletingLastPathComponent(), withIntermediateDirectories: true)
    try data.write(to: cache, options: .atomic)
    document = candidate; canvas.document = candidate
    canvas.navigation.fit(); status.text = "\(title) · \(document.strokeCount) \(document.strokeCount == 1 ? "stroke" : "strokes") · View Only"
    refreshAvailability()
  }
  private func sample() {
    cancelLoad()
    do { guard let url = Bundle.main.url(forResource: "Sample", withExtension: "nsketch") else { throw ViewerError.invalid }; try present(boundedRead(url, limit: DocumentDownload.limit), title: "Sample") }
    catch { status.text = "\(error.localizedDescription) Previous copy retained · View Only" }
  }
  private func openURL() {
    guard let url = URL(string: address.text ?? ""), url.scheme?.lowercased() == "https", url.host != nil, url.user == nil, url.password == nil else { status.text = "Enter a valid HTTPS URL · View Only"; return }
    cancelLoad(); let request = generation; status.text = "Loading copy… · View Only"
    download = DocumentDownload(url: url) { [weak self] result in
      Task { @MainActor in
        guard let self, self.generation == request else { return }
        self.download = nil
        defer { self.refreshAvailability() }
        do { try self.present(result.get(), title: url.lastPathComponent) }
        catch { self.status.text = "\(error.localizedDescription) Previous copy retained · View Only" }
      }
    }
    refreshAvailability()
  }
}
@MainActor final class TVScene: UIResponder, UIWindowSceneDelegate {
  var window: UIWindow?
  func scene(_ scene: UIScene, willConnectTo session: UISceneSession, options connectionOptions: UIScene.ConnectionOptions) {
    guard let scene = scene as? UIWindowScene else { return }; let window = UIWindow(windowScene: scene)
    window.rootViewController = TVController(); window.makeKeyAndVisible(); self.window = window
  }
}
@main @MainActor final class TVApp: UIResponder, UIApplicationDelegate {
  func application(_ application: UIApplication, configurationForConnecting session: UISceneSession, options: UIScene.ConnectionOptions) -> UISceneConfiguration {
    let configuration = UISceneConfiguration(name: nil, sessionRole: session.role); configuration.delegateClass = TVScene.self; return configuration
  }
}
