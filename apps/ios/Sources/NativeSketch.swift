import CSketchFFI
import UIKit
import UniformTypeIdentifiers

@main @MainActor final class NativeSketchApplication: UIResponder, UIApplicationDelegate {
  func application(
    _ application: UIApplication, configurationForConnecting session: UISceneSession,
    options: UIScene.ConnectionOptions
  ) -> UISceneConfiguration {
    let configuration = UISceneConfiguration(name: "Drawing", sessionRole: session.role)
    configuration.delegateClass = DrawingSceneDelegate.self
    return configuration
  }
}

@MainActor final class DrawingSceneDelegate: UIResponder, UIWindowSceneDelegate {
  var window: UIWindow?
  func scene(
    _ scene: UIScene, willConnectTo session: UISceneSession, options: UIScene.ConnectionOptions
  ) {
    guard let scene = scene as? UIWindowScene else { return }
    let window = UIWindow(windowScene: scene)
    window.rootViewController = UINavigationController(rootViewController: DrawingController())
    self.window = window
    window.makeKeyAndVisible()
  }
  func sceneWillResignActive(_ scene: UIScene) {
    (window?.rootViewController as? UINavigationController)?.viewControllers.compactMap {
      $0 as? DrawingController
    }.forEach { $0.inactive() }
  }
  func sceneDidDisconnect(_ scene: UIScene) {
    sceneWillResignActive(scene)
    window = nil
  }
}
